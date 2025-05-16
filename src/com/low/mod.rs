use super::ir::{self, Solution, VariableID};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunID(pub usize);

pub enum Expr {
    Int {
        val: i64,
    },
    Float {
        val: f64,
    },
    String {
        val: String,
    },
    Bool {
        val: bool,
    },
    Bundle {
        items: Box<[Expr]>,
    },
    Access {
        accessed: Box<Expr>,
        index: u8,
    },
    Block {
        label: Option<ir::LabelID>,
        stmts: Box<[Stmt]>,
        result: Box<Expr>,
        needs_frame: bool,
    },
    Variant {
        tag: i64,
        items: Box<[Expr]>,
    },
    Local {
        local: u8,
    },
    If {
        label: ir::LabelID,
        guard: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    While {
        label: ir::LabelID,
        guard: Box<Expr>,
        do_branch: Box<Stmt>,
        else_branch: Box<Expr>,
    },
    Loop {
        label: ir::LabelID,
        body: Box<Stmt>,
    },
    Fun {
        id: FunID,
        captured: Box<[u8]>,
    },
    Call {
        callee: Box<Expr>,
        args: Box<[Expr]>,
    },
    Unwrap {
        value: Box<Expr>,
        unwrapping: Unwrapping,
    },
    Match {
        scrutinee: Box<Expr>,
        decision: Box<Decision>,
        fallback: Box<Expr>,
    },
    Break {
        value: Box<Expr>,
        label: ir::LabelID,
    },
    Skip {
        label: ir::LabelID,
    },

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),

    Pow(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>),
    Ln(Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Asin(Box<Expr>),
    Acos(Box<Expr>),
    Atan(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
}

pub enum Decision {
    Failure,
    Success {
        expr: Box<Expr>,
    },
    Test {
        local: u8,
        pat: Box<Pat>,
        success: Box<Decision>,
        failure: Box<Decision>,
    },
}

impl Expr {
    fn unit() -> Self {
        Self::Bundle {
            items: Box::new([]),
        }
    }
}

pub enum Stmt {
    Nothing,
    Expr {
        expr: Box<Expr>,
    },
    Let {
        bindings: Box<[(Pat, Expr)]>,
    },
    Block {
        stmts: Box<[Stmt]>,
        needs_frame: bool,
    },
}

pub enum Pat {
    Discard,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Local(u8),
    Bundle(Box<[Pat]>),
    Variant(usize, Box<[Pat]>),
}

#[derive(Default, Clone)]
pub struct CaptureInfo {
    variables: Vec<ir::VariableID>,
    functions: HashMap<ir::VariableID, FunID>,
}

pub struct Function {
    pub name: String,
    pub id: FunID,
    pub args: Box<[Pat]>,
    pub expr: Expr,
    pub captured_count: usize,
}

pub struct Program {
    pub functions: Vec<Function>,
}

struct Work {
    name: String,
    id: FunID,
    recursive_fun_id: FunID,
    recursive_binding: Option<ir::VariableID>,
    signature: ir::Signature,
    expr: ir::Expr,
    solutions: Vec<Solution>,
}

#[derive(Clone, Debug)]
pub enum Unwrapping {
    Done,
    Bundle { index: usize, next: Box<Unwrapping> },
}

impl Unwrapping {
    fn index(self, i: usize) -> Self {
        Self::Bundle {
            index: i,
            next: Box::new(self),
        }
    }
}

struct AbstractionInfo {
    abstract_expr: ir::Expr,
    expr_solutions: Vec<ir::Solution>,
    unwrappings_by_binding: HashMap<ir::VariableID, Unwrapping>,
}

type SolutionMap = HashMap<usize, Vec<ir::Solution>>;

struct Lowerer {
    entities: ir::Entities,
    work: Vec<Work>,

    local_index: usize,
    local_by_var: HashMap<ir::VariableID, u8>,

    function_index: usize,
    current_fun_id: FunID,
    capture_info_by_fun_id: HashMap<FunID, CaptureInfo>,

    solutions: SolutionMap,

    abstractions: Vec<AbstractionInfo>,
    abstraction_key_by_var: HashMap<ir::VariableID, usize>,

    builtins: HashMap<ir::Builtin, FunID>,
}

impl Lowerer {
    fn new(entities: ir::Entities) -> Self {
        Self {
            entities,
            work: Vec::new(),

            local_index: 0,
            local_by_var: HashMap::new(),

            function_index: 0,
            current_fun_id: FunID(0),
            capture_info_by_fun_id: HashMap::new(),

            solutions: SolutionMap::default(),

            abstractions: Vec::new(),
            abstraction_key_by_var: HashMap::new(),

            builtins: HashMap::new(),
        }
    }

    fn next_function_id(&mut self) -> FunID {
        self.function_index += 1;
        FunID(self.function_index - 1)
    }

    fn lower_program(
        mut self,
        mut modules: Vec<ir::Module>,
        dependency_order: Vec<usize>,
    ) -> Program {
        let mut stmts = Vec::new();
        let mut solutions = Vec::new();
        for file_id in dependency_order {
            let file_stmts = std::mem::take(&mut modules[file_id].stmts);
            stmts.extend_from_slice(&file_stmts);
            solutions.append(&mut modules[file_id].solutions);
        }

        // build main function task
        use ir::Signature as Sig;
        self.add_work(
            "<main>".to_string(),
            None,
            None,
            ir::Expr::BlockUnlabelled {
                stmts: stmts.into(),
            },
            Sig::Args {
                args: Box::new([]),
                next: Box::new(Sig::Done),
            },
            CaptureInfo::default(),
            solutions,
        );

        let mut functions = Vec::new();
        while let Some(work) = self.work.pop() {
            let fun = self.lower_function_work(work);
            functions.push(fun);
        }

        Program { functions }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_work(
        &mut self,
        name: String,
        recursive_binding: Option<ir::VariableID>,
        recursive_fun_id: Option<FunID>,
        expr: ir::Expr,
        signature: ir::Signature,
        capture_info: CaptureInfo,
        solutions: Vec<ir::Solution>,
    ) -> FunID {
        let id = self.next_function_id();
        self.capture_info_by_fun_id.insert(id, capture_info);
        self.work.push(Work {
            name,
            id,
            recursive_binding,
            recursive_fun_id: recursive_fun_id.unwrap_or(id),
            expr,
            signature,
            solutions,
        });
        id
    }

    // either Var or AbstractVar
    fn solve_class_item_expr(&self, item_id: usize, constraint_id: usize) -> ir::Expr {
        let Some(solutions) = self.solutions.get(&constraint_id) else {
            panic!("unknown solution for constraint id '{constraint_id}'")
        };
        let solution = &solutions[0];
        let instance_info = self.entities.get_instance_info(solution.instance_id);
        let item_info = &instance_info.items[item_id];

        let var_id = item_info.binding;
        match item_info.is_concrete {
            true => ir::Expr::Var { id: var_id },
            false => ir::Expr::AbstractVar {
                id: var_id,
                constraint_id: solution.additional_constraint_id,
            },
        }
    }

    fn register_solutions(&mut self, solutions: Vec<ir::Solution>) -> SolutionMap {
        let orig = std::mem::take(&mut self.solutions);

        // construct the map: (constraint_id -> [solutions...])
        for mut solution in solutions {
            if let Some(constraint_id) = solution.trace.constraint_ids.pop() {
                self.solutions.entry(constraint_id).or_default();
                self.solutions
                    .get_mut(&constraint_id)
                    .unwrap()
                    .push(solution);
            }
        }

        orig
    }

    fn restore_solutions(&mut self, orig: SolutionMap) {
        self.solutions = orig;
    }

    fn rebuild_solutions(&self) -> Vec<ir::Solution> {
        Self::rebuild_solutions_from_map(&self.solutions)
    }

    fn rebuild_solutions_from_map(map: &SolutionMap) -> Vec<ir::Solution> {
        map.iter()
            .flat_map(|(id, solutions)| {
                solutions.iter().cloned().map(|mut solution| {
                    solution.trace.constraint_ids.push(*id);
                    solution
                })
            })
            .collect()
    }

    fn lower_function_work(&mut self, work: Work) -> Function {
        self.local_by_var.clear();
        self.local_index = 0;
        self.solutions = Default::default();

        use ir::Signature as S;
        let S::Args { args, next } = work.signature else {
            panic!("attempt to lower function work with invalid signature");
        };

        // collect bindings in current args
        let mut arg_bindings = Vec::new();
        for arg in &args {
            arg.collect_bindings(&mut arg_bindings);
        }

        // declare them
        let args = args
            .into_iter()
            .map(|arg| self.lower_pattern(arg))
            .collect();

        // register captured variables as new locals
        let captured_count = self.capture_info_by_fun_id[&work.id].variables.len();
        for captured_id in self.capture_info_by_fun_id[&work.id].variables.clone() {
            self.register_local(captured_id);
        }

        // if this was the last argment signature, we are done
        if let S::Done = &*next {
            self.register_solutions(work.solutions); // solutions

            // recursive function call
            self.current_fun_id = work.id;
            if let Some(rec_id) = work.recursive_binding {
                self.capture_info_by_fun_id
                    .get_mut(&work.id)
                    .unwrap()
                    .functions
                    .insert(rec_id, work.recursive_fun_id);
            }

            // lower the function body
            return Function {
                name: work.name,
                id: work.id,
                args,
                expr: self.lower_expression(work.expr),
                captured_count,
            };
        }

        // otherwise, current args are immediately captured...
        let mut capture_info = self.capture_info_by_fun_id[&work.id].clone();
        for arg_binding in arg_bindings {
            capture_info.variables.push(arg_binding);
        }

        // get the captured locals in this auxiliary function
        let captured_locals = self.get_captured_locals_from_info(&capture_info);
        let captured_count = captured_locals.len();

        let next_id = self.add_work(
            format!("{}'", work.name),
            work.recursive_binding,
            Some(work.id), // recursive function id points to first function
            work.expr,
            *next,
            capture_info,
            work.solutions,
        );

        Function {
            name: work.name,
            id: work.id,
            args,
            expr: Expr::Fun {
                id: next_id,
                captured: captured_locals,
            },
            captured_count,
        }
    }

    fn get_captured_locals_from_info(&self, info: &CaptureInfo) -> Box<[u8]> {
        info.variables
            .iter()
            .map(|id| self.local_by_var[id])
            .collect()
    }

    fn lower_statement(&mut self, stmt: ir::Stmt) -> Stmt {
        use ir::Stmt as S;
        match stmt {
            S::Missing => unreachable!("attempt to generate missing statement"),
            S::Nothing => unreachable!("attempt to generate 'nothing' statement"),
            S::Expr { expr, ty: _ } => self.lower_expression_statement(expr),
            S::Let {
                lhs,
                rhs,
                is_concrete,
                solutions,
            } => self.lower_let_statement(lhs, rhs, is_concrete, solutions),
            S::Have { stmts } => self.lower_have_statement(stmts),
        }
    }

    fn lower_statement_list(&mut self, stmts: impl IntoIterator<Item = ir::Stmt>) -> Vec<Stmt> {
        use ir::Stmt as S;
        stmts
            .into_iter()
            .filter_map(|stmt| match stmt {
                S::Missing => None,
                S::Nothing => None,
                stmt => Some(self.lower_statement(stmt)),
            })
            .collect()
    }

    fn lower_statement_block(
        &mut self,
        stmts: impl IntoIterator<Item = ir::Stmt>,
        label: Option<ir::LabelID>,
    ) -> Expr {
        let local_index_orig = self.local_index;
        let stmts = self.lower_statement_list(stmts).into();
        let needs_frame = self.local_index != local_index_orig;
        self.local_index = local_index_orig;

        Expr::Block {
            label,
            stmts,
            result: Box::new(Expr::unit()),
            needs_frame,
        }
    }

    fn lower_statement_block_as_statement(
        &mut self,
        stmts: impl IntoIterator<Item = ir::Stmt>,
    ) -> Stmt {
        let local_index_orig = self.local_index;
        let stmts = self.lower_statement_list(stmts).into();
        let needs_frame = self.local_index != local_index_orig;
        self.local_index = local_index_orig;

        Stmt::Block { stmts, needs_frame }
    }

    fn lower_expression_statement(&mut self, expr: ir::Expr) -> Stmt {
        Stmt::Expr {
            expr: Box::new(self.lower_expression(expr)),
        }
    }

    fn simplify_deconstruct(lhs: Pat, rhs: Expr, bindings: &mut Vec<(Pat, Expr)>) {
        match (lhs, rhs) {
            (Pat::Int(_), Expr::Int { .. }) => {}
            (Pat::Float(_), Expr::Float { .. }) => {}
            (Pat::String(_), Expr::String { .. }) => {}
            (Pat::Bool(_), Expr::Bool { .. }) => {}

            (Pat::Bundle(left_items), Expr::Bundle { items: right_items })
            | (
                Pat::Variant(_, left_items),
                Expr::Variant {
                    tag: _,
                    items: right_items,
                },
            ) => {
                for (left_item, right_item) in left_items.into_iter().zip(right_items) {
                    Self::simplify_deconstruct(left_item, right_item, bindings);
                }
            }

            (pat, expr) => bindings.push((pat, expr)),
        }
    }

    fn collect_unwrappings(
        prev: Unwrapping,
        pat: &ir::Pattern,
        map: &mut HashMap<ir::VariableID, Unwrapping>,
    ) {
        use ir::Pattern as P;
        match pat {
            P::Missing => {}
            P::Discard => {}
            P::Binding(id) => {
                map.insert(*id, prev);
            }
            P::Int(_) | P::Float(_) | P::String(_) | P::Bool(_) => {}
            P::Record(_, items) | P::Tuple(items) => {
                for (i, item) in items.iter().enumerate() {
                    Self::collect_unwrappings(prev.clone().index(i), item, map);
                }
            }
            P::Variant(_, _, None) => {}
            P::Variant(_, _, Some(items)) => {
                for (i, item) in items.iter().enumerate() {
                    Self::collect_unwrappings(prev.clone().index(i).index(2), item, map);
                }
            }
        }
    }

    fn lower_let_statement(
        &mut self,
        lhs: ir::Pattern,
        rhs: ir::Expr,
        is_concrete: bool,
        solutions: Vec<ir::Solution>,
    ) -> Stmt {
        if !is_concrete {
            let bindings = lhs.get_binding_ids();
            let mut unwrappings = HashMap::new();
            Self::collect_unwrappings(Unwrapping::Done, &lhs, &mut unwrappings);

            let abstraction_key = self.abstractions.len();
            self.abstractions.push(AbstractionInfo {
                abstract_expr: rhs,
                expr_solutions: solutions,
                unwrappings_by_binding: unwrappings,
            });

            for binding in bindings {
                self.abstraction_key_by_var.insert(binding, abstraction_key);
            }

            return Stmt::Nothing;
        }

        let orig = self.register_solutions(solutions);

        let pat = self.lower_pattern(lhs);
        let expr = self.lower_expression(rhs);

        self.restore_solutions(orig);

        let mut bindings = Vec::new();
        Self::simplify_deconstruct(pat, expr, &mut bindings);

        Stmt::Let {
            bindings: bindings.into(),
        }
    }

    fn lower_have_statement(&mut self, stmts: Box<[ir::Stmt]>) -> Stmt {
        let stmts = self.lower_statement_list(stmts);
        Stmt::Block {
            stmts: stmts.into(),
            needs_frame: false,
        }
    }

    fn register_local(&mut self, id: ir::VariableID) -> u8 {
        let local: u8 = self
            .local_index
            .try_into()
            .expect("function cannot have more than 255 variables");
        self.local_by_var.insert(id, local);
        self.local_index += 1;
        local
    }

    fn get_local(&self, id: ir::VariableID) -> u8 {
        match self.local_by_var.get(&id) {
            Some(local) => *local,
            None => {
                let info = self.entities.get_variable_info(id);
                panic!("unregistered variable '{}' (#{})", info.name, id.0)
            }
        }
    }

    fn lower_pattern(&mut self, pat: ir::Pattern) -> Pat {
        use ir::Pattern as P;
        match pat {
            P::Missing => unreachable!("attempt to lower missing pattern"),
            P::Discard => Pat::Discard,
            P::Binding(id) => Pat::Local(self.register_local(id)),
            P::Int(val) => Pat::Int(val),
            P::Float(val) => Pat::Float(val),
            P::String(val) => Pat::String(val),
            P::Bool(val) => Pat::Bool(val),
            P::Tuple(items) => Pat::Bundle(
                items
                    .into_iter()
                    .map(|item| self.lower_pattern(item))
                    .collect(),
            ),
            P::Variant(_, tag, None) => Pat::Variant(tag, Box::new([])),
            P::Variant(_, tag, Some(items)) => Pat::Variant(
                tag,
                items
                    .into_iter()
                    .map(|item| self.lower_pattern(item))
                    .collect(),
            ),
            P::Record(_, fields) => Pat::Bundle(
                fields
                    .into_iter()
                    .map(|item| self.lower_pattern(item))
                    .collect(),
            ),
        }
    }

    fn lower_expression(&mut self, expr: ir::Expr) -> Expr {
        use ir::Expr as E;
        match expr {
            E::Missing => unreachable!("attempt to lower missing expr"),
            E::Int { val } => Self::lower_int(val),
            E::Float { val } => Self::lower_float(val),
            E::String { val } => Self::lower_string(val),
            E::Bool { val } => Self::lower_bool(val),
            E::Var { id } => self.lower_variable(id),
            E::AbstractVar { id, constraint_id } => self.lower_abstract_variable(id, constraint_id),
            E::Tuple { items } => self.lower_small_bundle(items),
            E::Array { items } => self.lower_small_bundle(items),
            E::Block { stmts, label } => self.lower_block_expression(stmts, Some(label)),
            E::BlockUnlabelled { stmts } => self.lower_block_expression(stmts, None),
            E::Conditional {
                branches,
                is_exhaustive,
            } => self.lower_conditional(branches, is_exhaustive),
            E::Break { expr, label } => self.lower_break(label, expr),
            E::Skip { label } => self.lower_skip(label),
            E::Fun {
                name,
                recursive_binding,
                signature,
                expr,
            } => self.lower_fun(name, recursive_binding, *signature, *expr),
            E::Call { callee, args } => self.lower_call(*callee, args),
            E::Variant { tag, items } => self.lower_variant(tag, items),
            E::Record { fields } => self.lower_small_bundle(fields),
            E::Access { accessed, index } => self.lower_access(*accessed, index),
            E::ClassItem {
                item_id,
                constraint_id,
            } => {
                let item_expr = self.solve_class_item_expr(item_id, constraint_id);
                self.lower_expression(item_expr)
            }
            E::Builtin(builtin) => self.lower_builtin(builtin),

            E::Add(left, right) => Expr::Add(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Sub(left, right) => Expr::Sub(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Mul(left, right) => Expr::Mul(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Div(left, right) => Expr::Div(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Mod(left, right) => Expr::Mod(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::BitAnd(left, right) => Expr::BitAnd(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::BitOr(left, right) => Expr::BitOr(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::BitXor(left, right) => Expr::BitXor(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),

            E::Pow(left, right) => Expr::Pow(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Exp(arg) => Expr::Exp(Box::new(self.lower_expression(*arg))),
            E::Ln(arg) => Expr::Ln(Box::new(self.lower_expression(*arg))),
            E::Sin(arg) => Expr::Sin(Box::new(self.lower_expression(*arg))),
            E::Cos(arg) => Expr::Cos(Box::new(self.lower_expression(*arg))),
            E::Tan(arg) => Expr::Tan(Box::new(self.lower_expression(*arg))),
            E::Asin(arg) => Expr::Asin(Box::new(self.lower_expression(*arg))),
            E::Acos(arg) => Expr::Acos(Box::new(self.lower_expression(*arg))),
            E::Atan(arg) => Expr::Atan(Box::new(self.lower_expression(*arg))),

            E::Eq(left, right) => Expr::Eq(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Ne(left, right) => Expr::Ne(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Lt(left, right) => Expr::Lt(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Le(left, right) => Expr::Le(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Gt(left, right) => Expr::Gt(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
            E::Ge(left, right) => Expr::Ge(
                Box::new(self.lower_expression(*left)),
                Box::new(self.lower_expression(*right)),
            ),
        }
    }

    fn lower_int(val: i64) -> Expr {
        Expr::Int { val }
    }

    fn lower_float(val: f64) -> Expr {
        Expr::Float { val }
    }

    fn lower_string(val: String) -> Expr {
        Expr::String { val }
    }

    fn lower_bool(val: bool) -> Expr {
        Expr::Bool { val }
    }

    fn lower_variable(&mut self, id: ir::VariableID) -> Expr {
        // recursive function binding
        let capture_info = self.get_current_capture_info();
        if let Some(fun_id) = capture_info.functions.get(&id).copied() {
            let rec_capture_info = &self.capture_info_by_fun_id[&fun_id];
            return Expr::Fun {
                id: fun_id,
                captured: self.get_captured_locals_from_info(rec_capture_info),
            };
        }

        // regular local variable
        let local = self.get_local(id);
        Expr::Local { local }
    }

    fn get_variable_abstraction_info(&self, id: VariableID) -> &AbstractionInfo {
        let key = self
            .abstraction_key_by_var
            .get(&id)
            .copied()
            .expect("variable id is not abstract");
        self.abstractions
            .get(key)
            .expect("incorrect abstraction info key")
    }

    fn build_abstract_expression_solutions(
        &self,
        id: ir::VariableID,
        constraint_id: usize,
    ) -> Vec<ir::Solution> {
        let mut orig = self.solutions.clone();
        let info = self.get_variable_abstraction_info(id);

        let mut abstract_expr_solutions = info.expr_solutions.clone();
        let mut elaborated_solutions = orig
            .remove(&constraint_id)
            .expect("failed to elaborate solution")
            .clone();
        let mut irrelevant_solutions = Self::rebuild_solutions_from_map(&orig);

        let mut new_solutions = Vec::new();
        new_solutions.append(&mut abstract_expr_solutions);
        new_solutions.append(&mut elaborated_solutions);
        new_solutions.append(&mut irrelevant_solutions);
        new_solutions
    }

    fn lower_abstract_variable(&mut self, id: ir::VariableID, constraint_id: usize) -> Expr {
        let info = self.get_variable_abstraction_info(id);
        let abstract_expr = info.abstract_expr.clone();

        let abstract_expr_solutions = self.build_abstract_expression_solutions(id, constraint_id);

        let orig = self.register_solutions(abstract_expr_solutions);
        let expr = self.lower_expression(abstract_expr);
        self.restore_solutions(orig);

        // unwrap the correct binding from the abstract bundle
        let info = self.get_variable_abstraction_info(id);
        let unwrapping = info.unwrappings_by_binding[&id].clone();

        // emit the abstract variable and return the unwrapping expression
        Expr::Unwrap {
            value: Box::new(expr),
            unwrapping,
        }
    }

    fn lower_expression_list(&mut self, items: impl IntoIterator<Item = ir::Expr>) -> Vec<Expr> {
        items
            .into_iter()
            .map(|item| self.lower_expression(item))
            .collect()
    }

    fn lower_small_bundle(&mut self, items: Box<[ir::Expr]>) -> Expr {
        Expr::Bundle {
            items: self.lower_expression_list(items).into(),
        }
    }

    fn lower_access(&mut self, accessed: ir::Expr, index: usize) -> Expr {
        Expr::Access {
            accessed: Box::new(self.lower_expression(accessed)),
            index: index.try_into().expect("cannot index beyond 255"),
        }
    }

    fn lower_conditional(&mut self, branches: Box<[ir::Branch]>, is_exhaustive: bool) -> Expr {
        let mut fallback = Expr::unit();
        for branch in branches.into_iter().rev() {
            use ir::Branch as B;
            match branch {
                B::If { guard, body, label } => {
                    fallback = Expr::If {
                        label,
                        guard: Box::new(self.lower_expression(*guard)),
                        then_branch: match is_exhaustive {
                            true => Box::new(self.lower_block_expression(body, None)),
                            false => Box::new(self.lower_statement_block(body, None)),
                        },
                        else_branch: Box::new(fallback),
                    };
                }
                B::While { guard, body, label } => {
                    fallback = Expr::While {
                        label,
                        guard: Box::new(self.lower_expression(*guard)),
                        do_branch: Box::new(self.lower_statement_block_as_statement(body)),
                        else_branch: Box::new(fallback),
                    }
                }
                B::Loop { body, label } => {
                    fallback = Expr::Loop {
                        label,
                        body: Box::new(self.lower_statement_block_as_statement(body)),
                    }
                }
                B::Else { body, label } => {
                    fallback = self.lower_block_expression(body, Some(label))
                }
                B::Match {
                    scrutinee_var,
                    scrutinee,
                    decision,
                } => {
                    let scrutinee = self.lower_expression(*scrutinee);
                    self.register_local(scrutinee_var);
                    let decision = self.lower_decision(*decision, is_exhaustive);
                    fallback = Expr::Match {
                        scrutinee: Box::new(scrutinee),
                        decision: Box::new(decision),
                        fallback: Box::new(fallback),
                    };
                }
            }
        }

        fallback
    }

    fn lower_decision(&mut self, decision: ir::Decision, is_exhaustive: bool) -> Decision {
        use ir::Decision as D;
        match decision {
            D::Failure => Decision::Failure,
            D::Success { mut stmts, result } => {
                stmts.push(ir::Stmt::Expr {
                    expr: *result,
                    ty: ir::TypeID::whatever(),
                });

                Decision::Success {
                    expr: match is_exhaustive {
                        true => Box::new(self.lower_block_expression(stmts, None)),
                        false => Box::new(self.lower_statement_block(stmts, None)),
                    },
                }
            }
            D::Test {
                tested_var,
                pattern,
                success,
                failure,
            } => {
                let local = self.get_local(tested_var);

                // if the test is successful, the tested value is deconstructed
                // but if not, then no new locals must be declared
                // that's why the local index is restored after this branch is lowered
                // (basically, they are 'in parallel')
                let local_index_orig = self.local_index;
                let pat = self.lower_pattern(*pattern);
                let success = self.lower_decision(*success, is_exhaustive);
                self.local_index = local_index_orig;

                let failure = self.lower_decision(*failure, is_exhaustive);
                Decision::Test {
                    local,
                    pat: Box::new(pat),
                    success: Box::new(success),
                    failure: Box::new(failure),
                }
            }
        }
    }

    fn lower_break(&mut self, label: ir::LabelID, expr: Option<Box<ir::Expr>>) -> Expr {
        let expr = expr
            .map(|e| self.lower_expression(*e))
            .unwrap_or(Expr::unit());
        Expr::Break {
            value: Box::new(expr),
            label,
        }
    }

    fn lower_skip(&mut self, label: ir::LabelID) -> Expr {
        Expr::Skip { label }
    }

    fn lower_variant(&mut self, tag: usize, items: Option<Box<[ir::Expr]>>) -> Expr {
        Expr::Variant {
            tag: tag as i64,
            items: match items {
                Some(items) => self.lower_expression_list(items).into(),
                None => Box::new([]),
            },
        }
    }

    fn lower_block_expression(
        &mut self,
        stmts: impl IntoIterator<Item = ir::Stmt>,
        label: Option<ir::LabelID>,
    ) -> Expr {
        let local_index_orig = self.local_index;

        let mut stmts = self.lower_statement_list(stmts);
        let last = match stmts.pop() {
            Some(Stmt::Expr { expr }) => *expr,
            Some(stmt) => {
                stmts.push(stmt);
                Expr::unit()
            }
            None => Expr::unit(),
        };

        let needs_frame = self.local_index != local_index_orig;
        self.local_index = local_index_orig;

        Expr::Block {
            label,
            stmts: stmts.into(),
            result: Box::new(last),
            needs_frame,
        }
    }

    fn lower_fun(
        &mut self,
        name: String,
        recursive_binding: Option<ir::VariableID>,
        signature: ir::Signature,
        expr: ir::Expr,
    ) -> Expr {
        let mut captured = HashSet::new();
        let mut functions = HashMap::new();
        self.collect_expr_captured_variables(&expr, &mut captured, &mut functions);

        let capture_info = CaptureInfo {
            variables: captured.into_iter().collect(),
            functions,
        };
        let captured_locals = self.get_captured_locals_from_info(&capture_info);

        let solutions = self.rebuild_solutions();

        let id = self.add_work(
            name,
            recursive_binding,
            None,
            expr,
            signature,
            capture_info,
            solutions,
        );
        Expr::Fun {
            id,
            captured: captured_locals,
        }
    }

    fn get_current_capture_info(&self) -> &CaptureInfo {
        &self.capture_info_by_fun_id[&self.current_fun_id]
    }

    fn collect_stmt_captured_variables(
        &mut self,
        stmt: &ir::Stmt,
        set: &mut HashSet<ir::VariableID>,
        fun_map: &mut HashMap<ir::VariableID, FunID>,
    ) {
        use ir::Stmt as S;
        match stmt {
            S::Missing | S::Nothing => {}
            S::Expr { expr, ty: _ } => self.collect_expr_captured_variables(expr, set, fun_map),
            S::Let {
                lhs: _,
                rhs,
                is_concrete: _,
                solutions,
            } => {
                let orig = self.register_solutions(solutions.clone());
                self.collect_expr_captured_variables(rhs, set, fun_map);
                self.restore_solutions(orig);
            }
            S::Have { stmts } => {
                for stmt in stmts {
                    self.collect_stmt_captured_variables(stmt, set, fun_map);
                }
            }
        }
    }

    fn collect_expr_captured_variables(
        &mut self,
        expr: &ir::Expr,
        set: &mut HashSet<ir::VariableID>,
        fun_map: &mut HashMap<ir::VariableID, FunID>,
    ) {
        use ir::Expr as E;
        match expr {
            E::Missing | E::Int { .. } | E::Float { .. } | E::String { .. } | E::Bool { .. } => {}
            E::Var { id } => {
                if let Some(fun_id) = self.get_current_capture_info().functions.get(id) {
                    fun_map.insert(*id, *fun_id);
                }
                if self.local_by_var.contains_key(id) {
                    set.insert(*id);
                }
            }
            E::AbstractVar { id, constraint_id } => {
                let info = self.get_variable_abstraction_info(*id);
                let abstract_expr = info.abstract_expr.clone();

                let abstract_expr_solutions =
                    self.build_abstract_expression_solutions(*id, *constraint_id);

                let orig = self.register_solutions(abstract_expr_solutions);
                self.collect_expr_captured_variables(&abstract_expr, set, fun_map);
                self.restore_solutions(orig);
            }
            E::Tuple { items } | E::Array { items } => {
                for item in items {
                    self.collect_expr_captured_variables(item, set, fun_map);
                }
            }
            E::Block { stmts, label: _ } | E::BlockUnlabelled { stmts } => {
                for stmt in stmts {
                    self.collect_stmt_captured_variables(stmt, set, fun_map);
                }
            }
            E::Conditional {
                branches,
                is_exhaustive: _,
            } => {
                for branch in branches {
                    use ir::Branch as B;
                    match branch {
                        B::If {
                            guard,
                            body,
                            label: _,
                        }
                        | B::While {
                            guard,
                            body,
                            label: _,
                        } => {
                            self.collect_expr_captured_variables(guard, set, fun_map);
                            for stmt in body {
                                self.collect_stmt_captured_variables(stmt, set, fun_map);
                            }
                        }
                        B::Loop { body, label: _ } | B::Else { body, label: _ } => {
                            for stmt in body {
                                self.collect_stmt_captured_variables(stmt, set, fun_map);
                            }
                        }
                        B::Match {
                            scrutinee_var: _,
                            scrutinee: _,
                            decision,
                        } => self.collect_decision_captured_variables(decision, set, fun_map),
                    }
                }
            }
            E::Break {
                expr: Some(expr),
                label: _,
            } => self.collect_expr_captured_variables(expr, set, fun_map),
            E::Break { .. } | E::Skip { .. } => {}
            E::Fun {
                name: _,
                recursive_binding: _,
                signature: _,
                expr,
            } => self.collect_expr_captured_variables(expr, set, fun_map),
            E::Call { callee, args } => {
                self.collect_expr_captured_variables(callee, set, fun_map);
                for arg in args {
                    self.collect_expr_captured_variables(arg, set, fun_map);
                }
            }
            E::Variant { tag: _, items } => {
                if let Some(items) = items {
                    for item in items {
                        self.collect_expr_captured_variables(item, set, fun_map);
                    }
                }
            }
            E::Record { fields } => {
                for field in fields {
                    self.collect_expr_captured_variables(field, set, fun_map);
                }
            }
            E::Access { accessed, index: _ } => {
                self.collect_expr_captured_variables(accessed, set, fun_map);
            }
            E::ClassItem {
                item_id,
                constraint_id,
            } => {
                let item_expr = self.solve_class_item_expr(*item_id, *constraint_id);
                self.collect_expr_captured_variables(&item_expr, set, fun_map);
            }
            E::Builtin(..) => {}
            E::Add(left, right)
            | E::Sub(left, right)
            | E::Mul(left, right)
            | E::Div(left, right)
            | E::Mod(left, right)
            | E::BitAnd(left, right)
            | E::BitOr(left, right)
            | E::BitXor(left, right)
            | E::Eq(left, right)
            | E::Ne(left, right)
            | E::Lt(left, right)
            | E::Le(left, right)
            | E::Gt(left, right)
            | E::Ge(left, right)
            | E::Pow(left, right) => {
                self.collect_expr_captured_variables(left, set, fun_map);
                self.collect_expr_captured_variables(right, set, fun_map);
            }

            E::Exp(arg)
            | E::Ln(arg)
            | E::Sin(arg)
            | E::Cos(arg)
            | E::Tan(arg)
            | E::Asin(arg)
            | E::Acos(arg)
            | E::Atan(arg) => {
                self.collect_expr_captured_variables(arg, set, fun_map);
            }
        }
    }

    fn collect_decision_captured_variables(
        &mut self,
        decision: &ir::Decision,
        set: &mut HashSet<ir::VariableID>,
        fun_map: &mut HashMap<ir::VariableID, FunID>,
    ) {
        use ir::Decision as D;
        match decision {
            D::Failure => {}
            D::Success { stmts, result } => {
                for stmt in stmts {
                    self.collect_stmt_captured_variables(stmt, set, fun_map);
                }
                self.collect_expr_captured_variables(result, set, fun_map);
            }
            D::Test {
                tested_var: _,
                pattern: _,
                success,
                failure,
            } => {
                self.collect_decision_captured_variables(success, set, fun_map);
                self.collect_decision_captured_variables(failure, set, fun_map);
            }
        }
    }

    fn lower_call(&mut self, callee: ir::Expr, args: Box<[ir::Expr]>) -> Expr {
        Expr::Call {
            callee: Box::new(self.lower_expression(callee)),
            args: self.lower_expression_list(args).into(),
        }
    }

    fn lower_builtin(&mut self, builtin: ir::Builtin) -> Expr {
        let id = match self.builtins.get(&builtin) {
            Some(id) => *id,
            None => {
                let id = self.create_builtin_function_work(builtin);
                self.builtins.insert(builtin, id);
                id
            }
        };

        Expr::Fun {
            id,
            captured: Box::new([]),
        }
    }

    fn create_builtin_function_work(&mut self, builtin: ir::Builtin) -> FunID {
        use ir::Builtin as Bi;
        let (signature, expr) = match builtin {
            Bi::int_add => builtin_binary!(self, Add),
            Bi::int_sub => builtin_binary!(self, Sub),
            Bi::int_mul => builtin_binary!(self, Mul),
            Bi::int_div => builtin_binary!(self, Div),
            Bi::int_mod => builtin_binary!(self, Mod),
            Bi::int_and => builtin_binary!(self, BitAnd),
            Bi::int_or => builtin_binary!(self, BitOr),
            Bi::int_xor => builtin_binary!(self, BitXor),
            Bi::int_eq => builtin_binary!(self, Eq),
            Bi::int_ne => builtin_binary!(self, Ne),
            Bi::int_lt => builtin_binary!(self, Lt),
            Bi::int_le => builtin_binary!(self, Le),
            Bi::int_gt => builtin_binary!(self, Gt),
            Bi::int_ge => builtin_binary!(self, Ge),

            Bi::float_add => builtin_binary!(self, Add),
            Bi::float_sub => builtin_binary!(self, Sub),
            Bi::float_mul => builtin_binary!(self, Mul),
            Bi::float_div => builtin_binary!(self, Div),
            Bi::float_mod => builtin_binary!(self, Mod),
            Bi::float_eq => builtin_binary!(self, Eq),
            Bi::float_ne => builtin_binary!(self, Ne),
            Bi::float_lt => builtin_binary!(self, Lt),
            Bi::float_le => builtin_binary!(self, Le),
            Bi::float_gt => builtin_binary!(self, Gt),
            Bi::float_ge => builtin_binary!(self, Ge),

            Bi::string_concat => builtin_binary!(self, Add),
            Bi::string_eq => builtin_binary!(self, Eq),
            Bi::string_ne => builtin_binary!(self, Ne),
            Bi::string_lt => builtin_binary!(self, Lt),
            Bi::string_le => builtin_binary!(self, Le),
            Bi::string_gt => builtin_binary!(self, Gt),
            Bi::string_ge => builtin_binary!(self, Ge),

            Bi::bool_and => builtin_binary!(self, BitAnd),
            Bi::bool_or => builtin_binary!(self, BitOr),
            Bi::bool_xor => builtin_binary!(self, BitXor),
            Bi::bool_eq => builtin_binary!(self, Eq),
            Bi::bool_ne => builtin_binary!(self, Ne),

            Bi::pow => builtin_binary!(self, Pow),
            Bi::exp => builtin_unary!(self, Exp),
            Bi::ln => builtin_unary!(self, Ln),
            Bi::sin => builtin_unary!(self, Sin),
            Bi::cos => builtin_unary!(self, Cos),
            Bi::tan => builtin_unary!(self, Tan),
            Bi::asin => builtin_unary!(self, Asin),
            Bi::acos => builtin_unary!(self, Acos),
            Bi::atan => builtin_unary!(self, Atan),
        };

        self.add_work(
            builtin.to_string(),
            None,
            None,
            expr,
            signature,
            CaptureInfo::default(),
            Vec::new(),
        )
    }
}

pub fn lower(
    modules: Vec<ir::Module>,
    entities: ir::Entities,
    dependency_order: Vec<usize>,
) -> Program {
    Lowerer::new(entities).lower_program(modules, dependency_order)
}

macro_rules! builtin_unary {
    ($self:ident, $ctor:ident) => {{
        let arg = $self.entities.create_dummy_variable();
        (
            ir::Signature::Args {
                args: Box::new([ir::Pattern::Binding(arg)]),
                next: Box::new(ir::Signature::Done),
            },
            ir::Expr::$ctor(Box::new(ir::Expr::Var { id: arg })),
        )
    }};
}

macro_rules! builtin_binary {
    ($self:ident, $ctor:ident) => {{
        let left = $self.entities.create_dummy_variable();
        let right = $self.entities.create_dummy_variable();
        (
            ir::Signature::Args {
                args: Box::new([ir::Pattern::Binding(left), ir::Pattern::Binding(right)]),
                next: Box::new(ir::Signature::Done),
            },
            ir::Expr::$ctor(
                Box::new(ir::Expr::Var { id: left }),
                Box::new(ir::Expr::Var { id: right }),
            ),
        )
    }};
}

use builtin_binary;
use builtin_unary;

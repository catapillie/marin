use std::{collections::HashMap, i64};

use super::ir;

#[derive(Clone, Copy)]
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
    Block {
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
        guard: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    While {
        guard: Box<Expr>,
        do_branch: Box<Stmt>,
        else_branch: Box<Expr>,
    },
    Loop {
        body: Box<Stmt>,
    },
    Fun {
        id: FunID,
    },
    Call {
        callee: Box<Expr>,
        args: Box<[Expr]>,
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

pub struct Function {
    pub name: String,
    pub id: FunID,
    pub args: Box<[Pat]>,
    pub expr: Expr,
}

pub struct Program {
    pub functions: Vec<Function>,
}

struct Work {
    name: String,
    id: FunID,
    recursive_binding: Option<ir::VariableID>,
    signature: ir::Signature,
    expr: ir::Expr,
}

struct Lowerer {
    entities: ir::Entities,
    work: Vec<Work>,

    function_index: usize,
    local_index: usize,
    local_by_var: HashMap<ir::VariableID, u8>,
}

impl Lowerer {
    fn new(entities: ir::Entities) -> Self {
        Self {
            entities,
            work: Vec::new(),

            function_index: 0,
            local_index: 0,
            local_by_var: HashMap::new(),
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
        for file_id in dependency_order {
            let file_stmts = std::mem::take(&mut modules[file_id].stmts);
            stmts.extend_from_slice(&file_stmts);
        }

        // build main function task
        use ir::Signature as Sig;
        let id = self.next_function_id();
        self.work.push(Work {
            name: "<main>".to_string(),
            id,
            recursive_binding: None,
            expr: ir::Expr::BlockUnlabelled {
                stmts: stmts.into(),
            },
            signature: Sig::Args {
                args: Box::new([]),
                next: Box::new(Sig::Done),
            },
        });

        let mut functions = Vec::new();
        while let Some(work) = self.work.pop() {
            let fun = self.lower_function_work(work);
            functions.push(fun);
        }

        Program { functions }
    }

    fn lower_function_work(&mut self, work: Work) -> Function {
        self.local_by_var.clear();
        self.local_index = 0;

        use ir::Signature as S;
        let S::Args { args, next } = work.signature else {
            panic!("attempt to lower function work with invalid signature");
        };

        let args = args
            .into_iter()
            .map(|arg| self.lower_pattern(arg))
            .collect();

        if let S::Done = &*next {
            return Function {
                name: work.name,
                id: work.id,
                args,
                expr: self.lower_expression(work.expr),
            };
        }

        let next_id = self.next_function_id();
        self.work.push(Work {
            name: format!("{}'", work.name),
            id: next_id,
            recursive_binding: None,
            signature: *next,
            expr: work.expr,
        });

        Function {
            name: work.name,
            id: work.id,
            args,
            expr: Expr::Fun { id: next_id },
        }
    }

    fn lower_statement(&mut self, stmt: ir::Stmt) -> Stmt {
        use ir::Stmt as S;
        match stmt {
            S::Missing => unreachable!("attempt to generate missing statement"),
            S::Nothing => unreachable!("attempt to generate 'nothing' statement"),
            S::Expr { expr, ty: _ } => self.lower_expression_statement(expr),
            S::Let { lhs, rhs } => self.lower_let_statement(lhs, rhs),
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

    fn lower_statement_block(&mut self, stmts: impl IntoIterator<Item = ir::Stmt>) -> Expr {
        let local_index_orig = self.local_index;
        let stmts = self.lower_statement_list(stmts).into();
        let needs_frame = self.local_index != local_index_orig;
        self.local_index = local_index_orig;

        Expr::Block {
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

    fn lower_let_statement(&mut self, lhs: ir::Pattern, rhs: ir::Expr) -> Stmt {
        let pat = self.lower_pattern(lhs);
        let expr = self.lower_expression(rhs);

        let mut bindings = Vec::new();
        Self::simplify_deconstruct(pat, expr, &mut bindings);

        Stmt::Let {
            bindings: bindings.into(),
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
            None => panic!("unregistered variable '{}'", id.0),
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
            E::Var { id } => self.lower_local(id),
            E::Tuple { items } => self.lower_small_bundle(items),
            E::Array { items } => self.lower_small_bundle(items),
            E::Block { stmts, label } => self.lower_block_expression(stmts),
            E::BlockUnlabelled { stmts } => self.lower_block_expression(stmts),
            E::Conditional {
                branches,
                is_exhaustive,
            } => self.lower_conditional(branches, is_exhaustive),
            E::Break { expr, label } => todo!(),
            E::Skip { label } => todo!(),
            E::Fun {
                name,
                recursive_binding,
                signature,
                expr,
            } => self.lower_fun(name, recursive_binding, *signature, *expr),
            E::Call { callee, args } => self.lower_call(*callee, args),
            E::Variant { tag, items } => self.lower_variant(tag, items),
            E::Record { fields } => self.lower_small_bundle(fields),
            E::Access { accessed, index } => todo!(),
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

    fn lower_local(&self, id: ir::VariableID) -> Expr {
        let local = self.get_local(id);
        Expr::Local { local }
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

    fn lower_conditional(&mut self, branches: Box<[ir::Branch]>, is_exhaustive: bool) -> Expr {
        let mut fallback = Expr::unit();
        for branch in branches.into_iter().rev() {
            use ir::Branch as B;
            match branch {
                B::If { guard, body, label } => {
                    fallback = Expr::If {
                        guard: Box::new(self.lower_expression(*guard)),
                        then_branch: match is_exhaustive {
                            true => Box::new(self.lower_block_expression(body)),
                            false => Box::new(self.lower_statement_block(body)),
                        },
                        else_branch: Box::new(fallback),
                    };
                }
                B::While { guard, body, label } => {
                    fallback = Expr::While {
                        guard: Box::new(self.lower_expression(*guard)),
                        do_branch: Box::new(self.lower_statement_block_as_statement(body)),
                        else_branch: Box::new(fallback),
                    }
                }
                B::Loop { body, label } => {
                    fallback = Expr::Loop {
                        body: Box::new(self.lower_statement_block_as_statement(body)),
                    }
                }
                B::Else { body, label } => fallback = self.lower_block_expression(body),
                B::Match {
                    scrutinee_var,
                    scrutinee,
                    decision,
                } => todo!(),
            }
        }

        fallback
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

    fn lower_block_expression(&mut self, stmts: impl IntoIterator<Item = ir::Stmt>) -> Expr {
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
        let id = self.next_function_id();
        self.work.push(Work {
            name,
            id,
            recursive_binding,
            signature,
            expr,
        });
        Expr::Fun { id }
    }

    fn lower_call(&mut self, callee: ir::Expr, args: Box<[ir::Expr]>) -> Expr {
        Expr::Call {
            callee: Box::new(self.lower_expression(callee)),
            args: self.lower_expression_list(args).into(),
        }
    }
}

pub fn lower(
    modules: Vec<ir::Module>,
    entities: ir::Entities,
    dependency_order: Vec<usize>,
) -> Program {
    Lowerer::new(entities).lower_program(modules, dependency_order)
}

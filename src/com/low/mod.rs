use std::sync::Arc;

use super::ir;

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
    Tuple {
        items: Box<[Expr]>,
    },
    Array {
        items: Box<[Expr]>,
    },
    Block {
        stmts: Box<[Stmt]>,
        result: Box<Expr>,
    },
}

impl Expr {
    fn unit() -> Self {
        Self::Tuple {
            items: Box::new([]),
        }
    }
}

pub enum Stmt {
    Expr { expr: Box<Expr> },
}

pub enum Pat {
    Binding(),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Pat]>),
    Variant(usize, Option<Box<[Pat]>>),
}

pub struct Function {
    pub name: String,
    pub expr: Box<Expr>,
}

pub struct Program {
    pub functions: Vec<Function>,
}

struct Work {
    signature: ir::Signature,
}

struct Lowerer {
    entities: ir::Entities,
    work: Vec<Work>,
}

impl Lowerer {
    fn new(entities: ir::Entities) -> Self {
        Self {
            entities,
            work: Vec::new(),
        }
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

        let main_function = Function {
            name: "<main>".to_string(),
            expr: Box::new(self.lower_block_expression(stmts)),
        };
        let mut functions = vec![main_function];

        while let Some(work) = self.work.pop() {
            let fun = self.lower_function_work(work);
            functions.push(fun);
        }

        Program { functions }
    }

    fn lower_function_work(&mut self, work: Work) -> Function {
        todo!()
    }

    fn lower_statement(&mut self, stmt: ir::Stmt) -> Stmt {
        use ir::Stmt as S;
        match stmt {
            S::Missing => unreachable!("attempt to generate missing statement"),
            S::Nothing => unreachable!("attempt to generate 'nothing' statement"),
            S::Expr { expr, ty: _ } => self.lower_expression_statement(expr),
            S::Let { lhs, rhs } => todo!("lower let"),
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
        Expr::Block {
            stmts: self.lower_statement_list(stmts).into(),
            result: Box::new(Expr::unit()),
        }
    }

    fn lower_expression_statement(&mut self, expr: ir::Expr) -> Stmt {
        Stmt::Expr {
            expr: Box::new(self.lower_expression(expr)),
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
            E::Var { id } => todo!(),
            E::Tuple { items } => self.lower_tuple(items),
            E::Array { items } => self.lower_array(items),
            E::Block { stmts, label } => self.lower_block_expression(stmts),
            E::Conditional {
                branches,
                is_exhaustive,
            } => todo!(),
            E::Break { expr, label } => todo!(),
            E::Skip { label } => todo!(),
            E::Fun {
                name,
                recursive_binding,
                signature,
                expr,
            } => todo!(),
            E::Call { callee, args } => todo!(),
            E::Variant { tag, items } => todo!(),
            E::Record { fields } => todo!(),
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

    fn lower_tuple(&mut self, items: Box<[ir::Expr]>) -> Expr {
        Expr::Tuple {
            items: items
                .into_iter()
                .map(|item| self.lower_expression(item))
                .collect(),
        }
    }

    fn lower_array(&mut self, items: Box<[ir::Expr]>) -> Expr {
        Expr::Array {
            items: items
                .into_iter()
                .map(|item| self.lower_expression(item))
                .collect(),
        }
    }

    fn lower_block_expression(&mut self, stmts: impl IntoIterator<Item = ir::Stmt>) -> Expr {
        let mut stmts = self.lower_statement_list(stmts);

        let last = match stmts.pop() {
            Some(Stmt::Expr { expr }) => *expr,
            None => Expr::unit(),
        };

        Expr::Block {
            stmts: stmts.into(),
            result: Box::new(last),
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

use super::{Expr, Pattern, Solution, TypeID};

#[derive(Debug, Clone)]
pub enum Stmt {
    Missing,
    Nothing,
    Expr {
        expr: Expr,
        ty: TypeID,
    },
    Let {
        lhs: Pattern,
        rhs: Expr,
        is_concrete: bool,
        solutions: Vec<Solution>,
    },
    Have {
        stmts: Box<[Stmt]>,
    },
}

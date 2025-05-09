use super::{Expr, Pattern, TypeID};

#[derive(Debug, Clone)]
pub enum Stmt {
    Missing,
    Nothing,
    Expr { expr: Expr, ty: TypeID },
    Let { lhs: Pattern, rhs: Expr },
}

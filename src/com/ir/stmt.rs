use super::{Expr, Pattern, TypeID};

#[derive(Debug, Clone)]
pub enum Stmt {
    Missing,
    Nothing,
    Expr(Expr, TypeID),
    Let(Pattern, Expr),
}

use super::{Expr, Pattern, TypeID};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, TypeID),
    Let(Pattern, Expr),
}

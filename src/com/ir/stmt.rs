use super::{Expr, Pattern, TypeID};

#[derive(Debug)]
pub enum Stmt {
    Missing,
    Nothing,
    Expr(Expr, TypeID),
    Let(Pattern, Expr),
}

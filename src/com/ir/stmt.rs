use super::{Expr, Pattern, TypeID};

#[derive(Debug)]
pub enum Stmt {
    Missing,
    Expr(Expr, TypeID),
    Let(Pattern, Expr),
}

use super::{Expr, Pattern, TypeID};

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr, TypeID),
    Let(Pattern, Expr),
}

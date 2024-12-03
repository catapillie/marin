use super::{Expr, TypeID};

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr, TypeID),
    Let,
}

use super::{Expr, Pattern, Stmt, VariableID};

#[derive(Debug, Clone)]
pub enum Decision {
    Failure,
    Success(Box<[Stmt]>, Box<Expr>),
    Test(VariableID, Box<Pattern>, Box<Decision>, Box<Decision>),
}

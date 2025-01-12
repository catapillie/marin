use super::{EntityID, Expr, Pattern, Stmt};

#[derive(Debug, Clone)]
pub enum Decision {
    Failure,
    Success(Box<[Stmt]>, Box<Expr>),
    Test(EntityID, Box<Pattern>, Box<Decision>, Box<Decision>),
}

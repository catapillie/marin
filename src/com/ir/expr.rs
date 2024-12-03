use super::{EntityID, Stmt, TypeID};

pub type CheckedExpr = (Expr, TypeID);

#[derive(Debug)]
pub enum Expr {
    Missing,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Var(EntityID),
    Tuple(Box<[Expr]>),
    Array(Box<[Expr]>),
    Block(Box<[Stmt]>, Box<Expr>),
}

impl Expr {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

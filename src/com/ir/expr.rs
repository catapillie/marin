use super::{Branch, EntityID, LabelID, Stmt, TypeID};

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
    Block(Box<[Stmt]>, LabelID),
    Conditional(Box<[Branch]>, bool),
    Break(Option<Box<Expr>>, LabelID),
    Skip(LabelID),
}

impl Expr {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

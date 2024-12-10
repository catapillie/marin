use super::{EntityID, LabelID, Stmt, TypeID};

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
    Loop(Box<[Stmt]>, LabelID),
    Break(Option<Box<Expr>>, LabelID),
}

impl Expr {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

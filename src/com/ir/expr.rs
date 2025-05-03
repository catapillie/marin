use super::{Branch, EntityID, FunInfo, LabelID, Signature, Stmt, TypeID};

pub type CheckedExpr = (Expr, TypeID);

#[derive(Debug, Clone)]
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
    Fun(String, Option<EntityID>, FunInfo, Box<Signature>, Box<Expr>),
    Call(Box<Expr>, Box<[Expr]>),
    Variant(usize, Option<Box<[Expr]>>),
    Record(Box<[Expr]>),
    Access(Box<Expr>, usize),
}

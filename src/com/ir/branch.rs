use super::{Expr, LabelID, Stmt};

#[derive(Debug)]
pub enum Branch {
    If(Box<Expr>, Box<[Stmt]>, LabelID),
    While(Box<Expr>, Box<[Stmt]>, LabelID),
    Else(Box<[Stmt]>, LabelID),
}

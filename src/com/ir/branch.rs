use super::{Decision, Expr, LabelID, Stmt, VariableID};

#[derive(Debug, Clone)]
pub enum Branch {
    If(Box<Expr>, Box<[Stmt]>, LabelID),
    While(Box<Expr>, Box<[Stmt]>, LabelID),
    Loop(Box<[Stmt]>, LabelID),
    Else(Box<[Stmt]>, LabelID),
    Match(VariableID, Box<Expr>, Box<Decision>),
}

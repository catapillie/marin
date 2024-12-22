use super::{Expr, LabelID, Pattern, Stmt};

#[derive(Debug, Clone)]
pub enum Branch {
    If(Box<Expr>, Box<[Stmt]>, LabelID),
    While(Box<Expr>, Box<[Stmt]>, LabelID),
    Loop(Box<[Stmt]>, LabelID),
    Else(Box<[Stmt]>, LabelID),
    Match(Box<Expr>, Box<[(Pattern, Expr)]>),
}

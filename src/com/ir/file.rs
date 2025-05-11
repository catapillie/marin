use super::{Solution, Stmt};

pub struct Module {
    pub stmts: Box<[Stmt]>,
    pub solutions: Vec<Solution>,
}

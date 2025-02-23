use super::Stmt;

pub struct Module {
    pub stmts: Box<[Stmt]>,
}

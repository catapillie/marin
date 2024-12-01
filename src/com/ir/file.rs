use super::Stmt;

pub struct File {
    pub stmts: Box<[Stmt]>,
}

use super::{Expr, Pattern, Stmt, VariableID};

#[derive(Debug, Clone)]
pub enum Decision {
    Failure,
    Success {
        stmts: Vec<Stmt>,
        result: Box<Expr>,
    },
    Test {
        tested_var: VariableID,
        pattern: Box<Pattern>,
        success: Box<Decision>,
        failure: Box<Decision>,
    },
}

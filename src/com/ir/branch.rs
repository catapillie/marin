use super::{Decision, Expr, LabelID, Stmt, VariableID};

#[derive(Debug, Clone)]
pub enum Branch {
    If {
        guard: Box<Expr>,
        body: Box<[Stmt]>,
        label: LabelID,
    },
    While {
        guard: Box<Expr>,
        body: Box<[Stmt]>,
        label: LabelID,
    },
    Loop {
        body: Box<[Stmt]>,
        label: LabelID,
    },
    Else {
        body: Box<[Stmt]>,
        label: LabelID,
    },
    Match {
        scrutinee_var: VariableID,
        scrutinee: Box<Expr>,
        decision: Box<Decision>,
    },
}

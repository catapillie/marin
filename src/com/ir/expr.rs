use super::{Branch, ClassID, LabelID, Signature, Stmt, TypeID, VariableID};

pub type CheckedExpr = (Expr, TypeID);

#[derive(Debug, Clone)]
pub enum Expr {
    Missing,
    Int {
        val: i64,
    },
    Float {
        val: f64,
    },
    String {
        val: String,
    },
    Bool {
        val: bool,
    },
    Var {
        id: VariableID,
    },
    Tuple {
        items: Box<[Expr]>,
    },
    Array {
        items: Box<[Expr]>,
    },
    Block {
        stmts: Box<[Stmt]>,
        label: LabelID,
    },
    BlockUnlabelled {
        stmts: Box<[Stmt]>,
    },
    Conditional {
        branches: Box<[Branch]>,
        is_exhaustive: bool,
    },
    Break {
        expr: Option<Box<Expr>>,
        label: LabelID,
    },
    Skip {
        label: LabelID,
    },
    Fun {
        name: String,
        recursive_binding: Option<VariableID>,
        signature: Box<Signature>,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Box<[Expr]>,
    },
    Variant {
        tag: usize,
        items: Option<Box<[Expr]>>,
    },
    Record {
        fields: Box<[Expr]>,
    },
    Access {
        accessed: Box<Expr>,
        index: usize,
    },
    ClassItem {
        item_id: usize,
        constraint_id: usize,
    },
}

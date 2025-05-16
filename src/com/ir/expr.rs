use super::{Branch, Builtin, LabelID, Signature, Stmt, TypeID, VariableID};

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
    AbstractVar {
        id: VariableID,
        constraint_id: usize,
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
    Builtin(Builtin),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),

    Pos(Box<Expr>),
    Neg(Box<Expr>),
    BitNeg(Box<Expr>),

    Pow(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>),
    Ln(Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Asin(Box<Expr>),
    Acos(Box<Expr>),
    Atan(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
}

use super::{Expr, InstanceID, Pattern, Solution, TypeID, VariableID};

#[derive(Debug, Clone)]
pub enum Stmt {
    Missing,
    Nothing,
    Expr {
        expr: Expr,
        ty: TypeID,
    },
    Let {
        lhs: Pattern,
        rhs: Expr,
        is_concrete: bool,
        solutions: Vec<Solution>,
    },
    Have {
        instance_id: InstanceID,
        stmts: Box<[Stmt]>,
        item_bindings: Box<[VariableID]>,
    },
}

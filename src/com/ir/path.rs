use super::{CheckedExpr, EntityID, TypeID};

#[derive(Clone)]
pub enum PathQuery {
    Missing,
    Expr(CheckedExpr),
    Var(EntityID),
    Type(TypeID),
    Record(EntityID),
    Union(EntityID),
    Variant(EntityID, usize),
    Class(EntityID),
    ClassItem(EntityID, usize),
    Import(EntityID),
}

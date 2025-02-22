use super::{CheckedExpr, EntityID, TypeID};

pub enum PathQuery {
    Missing,
    Expr(CheckedExpr),
    Var(EntityID),
    Type(TypeID),
    Record(EntityID),
    Union(EntityID),
    Variant(EntityID, usize),
    Class(EntityID),
    Import(EntityID),
}

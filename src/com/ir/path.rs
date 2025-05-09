use super::{CheckedExpr, ClassID, ImportID, RecordID, TypeID, UnionID, VariableID};

#[derive(Clone, Debug)]
pub enum PathQuery {
    Missing,
    Expr(CheckedExpr),
    Var(VariableID),
    Type(TypeID),
    Record(RecordID),
    Union(UnionID),
    Variant(UnionID, usize),
    Class(ClassID),
    ClassItem(ClassID, usize),
    Import(ImportID),
}

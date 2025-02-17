use super::{Expr, Scheme, TypeID};
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntityID(pub usize);

pub enum Entity {
    Dummy,
    Variable(Variable),
    Type(TypeID),
    Record(RecordInfo),
    Union(UnionInfo),
    Class(ClassInfo),
}

pub struct Variable {
    pub name: String,
    pub scheme: Scheme,
    pub loc: Loc,
}

pub struct RecordInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
    pub type_args: Option<Box<[RecordArgInfo]>>,
    pub fields: Box<[RecordFieldInfo]>,
}

pub struct RecordArgInfo {
    #[allow(dead_code)]
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct RecordFieldInfo {
    pub name: String,
    pub ty: TypeID,
    pub loc: Loc,
}

pub struct UnionInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
    pub type_args: Option<Box<[UnionArgInfo]>>,
    pub variants: Box<[VariantInfo]>,
}

impl UnionInfo {
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

pub struct UnionArgInfo {
    #[allow(dead_code)]
    pub name: Option<String>,
}

pub struct VariantInfo {
    pub name: String,
    pub loc: Loc,
    pub expr: Expr,
    pub scheme: Scheme,
    pub type_args: Option<Box<[TypeID]>>,
}

impl VariantInfo {
    pub fn arity(&self) -> Option<usize> {
        self.type_args.as_ref().map(|args| args.len())
    }
}

pub struct ClassInfo {
    pub name: String,
    pub loc: Loc,
    pub items: Box<[ClassItemInfo]>,
    pub arity: (usize, usize),
}

pub struct ClassItemInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
}

use super::{Expr, Scheme, TypeID};
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntityID(pub usize);

pub enum Entity {
    Dummy,
    Variable(Variable),
    Type(TypeInfo),
}

pub struct Variable {
    pub scheme: Scheme,
    pub loc: Loc,
}

pub enum TypeInfo {
    Type(TypeID),
    Union(UnionInfo),
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

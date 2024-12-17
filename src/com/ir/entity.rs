use super::{Scheme, TypeID};
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone)]
pub struct EntityID(pub usize);

pub enum Entity {
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
    pub type_args: Option<Box<[UnionTypeArg]>>,
}

pub struct UnionTypeArg {
    #[allow(dead_code)]
    pub name: Option<String>,
}

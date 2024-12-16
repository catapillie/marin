use super::{Scheme, Type};
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone)]
pub struct EntityID(pub usize);

pub enum Entity {
    Variable(Variable),
    Type(TypeInfo)
}

pub struct Variable {
    pub scheme: Scheme,
    pub loc: Loc,
}

pub enum TypeInfo {
    TypeNode(Type)
}

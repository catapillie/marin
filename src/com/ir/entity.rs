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
}

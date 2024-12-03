use super::TypeID;
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone)]
pub struct EntityID(pub usize);

pub enum Entity {
    Variable(Variable),
    Type,
}

pub struct Variable {
    pub ty: TypeID,
    pub loc: Loc,
}

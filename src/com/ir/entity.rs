use super::Scheme;
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone)]
pub struct EntityID(pub usize);

pub enum Entity {
    Variable(Variable),
}

pub struct Variable {
    pub scheme: Scheme,
    pub loc: Loc,
}

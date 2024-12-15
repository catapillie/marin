use super::TypeID;
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LabelID(pub usize);

pub struct Label {
    pub name: Option<String>,
    pub ty: TypeID,
    pub skippable: bool,
    pub loc: Loc,
}

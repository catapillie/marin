use std::collections::BTreeSet;

use super::EntityID;

#[derive(Default, Debug, Clone)]
pub struct FunInfo {
    pub depth: usize,
    pub captured: BTreeSet<EntityID>,
}

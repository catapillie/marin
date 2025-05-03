use std::collections::HashSet;

use super::EntityID;

#[derive(Default, Debug, Clone)]
pub struct FunInfo {
    pub depth: usize,
    pub captured: HashSet<EntityID>,
}

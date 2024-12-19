use crate::com::{ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn next_entity_id(&self) -> ir::EntityID {
        ir::EntityID(self.entities.len())
    }

    pub fn create_entity(&mut self, entity: ir::Entity) -> ir::EntityID {
        let id = self.next_entity_id();
        self.entities.push(entity);
        id
    }

    pub fn get_entity(&self, id: ir::EntityID) -> &ir::Entity {
        &self.entities[id.0]
    }
}

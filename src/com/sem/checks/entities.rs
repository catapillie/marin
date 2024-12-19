use crate::com::{ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn create_entity(&mut self, entity: ir::Entity) -> ir::EntityID {
        let id = self.entities.len();
        self.entities.push(entity);
        ir::EntityID(id)
    }

    pub fn get_entity(&self, id: ir::EntityID) -> &ir::Entity {
        &self.entities[id.0]
    }
}

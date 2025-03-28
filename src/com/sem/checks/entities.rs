use colored::Colorize;

use crate::com::{ir, Checker};

impl Checker<'_, '_> {
    pub fn next_entity_id(&self) -> ir::EntityID {
        ir::EntityID(self.entities.len())
    }

    // only function allowed to grow 'self.entities' and 'self.entity_public'
    pub fn create_entity(&mut self, entity: ir::Entity) -> ir::EntityID {
        let id = self.next_entity_id();
        self.entities.push(entity);
        self.entity_public.push(false);
        id
    }

    pub fn create_entity_dummy(&mut self) -> ir::EntityID {
        self.create_entity(ir::Entity::Dummy)
    }

    pub fn get_entity(&self, id: ir::EntityID) -> &ir::Entity {
        &self.entities[id.0]
    }

    pub fn get_entity_mut(&mut self, id: ir::EntityID) -> &mut ir::Entity {
        &mut self.entities[id.0]
    }

    pub fn is_entity_public(&self, id: ir::EntityID) -> bool {
        self.entity_public[id.0]
    }

    pub fn set_entity_public(&mut self, id: ir::EntityID, public: bool) {
        self.entity_public[id.0] = public;
    }

    #[allow(dead_code)]
    pub fn get_entity_display(&self, id: ir::EntityID) -> String {
        use ir::Entity as Ent;
        match &self.entities[id.0] {
            Ent::Dummy => format!("({})", "dummy".bold()),
            Ent::Variable(info) => format!("({}) {}", "variable".bold(), info.name),
            Ent::Type(id) => format!("({}) {}", "type".bold(), self.get_type_string(*id)),
            Ent::Record(info) => format!("({}) {}", "record".bold(), info.name),
            Ent::Union(info) => format!("({}) {}", "union".bold(), info.name),
            Ent::Class(info) => format!("({}) {}", "class".bold(), info.name),
            Ent::Instance(info) => format!(
                "({}) {}",
                "instance".bold(),
                self.get_instance_scheme_string(&info.scheme)
            ),
            Ent::Import(info) => format!("({}) {}", "import".bold(), info.name),
            Ent::Alias(info) => format!("({}) {}", "alias".bold(), info.name),
        }
    }
}

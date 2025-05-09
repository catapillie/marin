use crate::com::{ir, Checker};
use colored::Colorize;

impl Checker<'_, '_> {
    pub fn is_entity_public(&self, id: ir::AnyID) -> bool {
        self.publics.contains(&id)
    }

    pub fn set_entity_public(&mut self, id: ir::AnyID, public: bool) {
        match public {
            true => self.publics.insert(id),
            false => self.publics.remove(&id),
        };
    }

    pub fn get_entity_display(&self, id: ir::AnyID) -> String {
        use ir::AnyID as ID;
        match id {
            ID::Variable(id) => {
                let info = self.entities.get_variable_info(id);
                format!("({}) {}", "variable".bold(), info.name)
            }
            ID::UserType(id) => {
                let info = self.entities.get_user_type_info(id);
                format!("({}) {}", "type".bold(), self.get_type_string(info.id))
            }
            ID::Record(id) => {
                let info = self.entities.get_record_info(id);
                format!("({}) {}", "record".bold(), info.name)
            }
            ID::Union(id) => {
                let info = self.entities.get_union_info(id);
                format!("({}) {}", "union".bold(), info.name)
            }
            ID::Class(id) => {
                let info = self.entities.get_class_info(id);
                format!("({}) {}", "class".bold(), info.name)
            }
            ID::Instance(id) => {
                let info = self.entities.get_instance_info(id);
                format!(
                    "({}) {}",
                    "instance".bold(),
                    self.get_instance_scheme_string(&info.scheme)
                )
            }
            ID::Import(id) => {
                let info = self.entities.get_import_info(id);
                format!("({}) {}", "import".bold(), info.name)
            }
            ID::Alias(id) => {
                let info = self.entities.get_alias_info(id);
                format!("({}) {}", "alias".bold(), info.name)
            }
        }
    }
}

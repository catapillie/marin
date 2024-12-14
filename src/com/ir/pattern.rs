use super::EntityID;

#[derive(Debug)]
pub enum Pattern {
    Missing,
    Binding(EntityID),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Pattern]>),
}

impl Pattern {
    fn collect_bindings(&self, bindings: &mut Vec<EntityID>) {
        match self {
            Self::Missing => {}
            Self::Int(_) => {}
            Self::Float(_) => {}
            Self::String(_) => {}
            Self::Bool(_) => {}
            Self::Binding(id) => bindings.push(*id),
            Self::Tuple(items) => {
                for item in items {
                    item.collect_bindings(bindings);
                }
            }
        }
    }

    pub fn get_binding_ids(&self) -> Vec<EntityID> {
        let mut bindings = Vec::new();
        self.collect_bindings(&mut bindings);
        bindings
    }
}

use super::EntityID;

#[derive(Debug, Clone)]
pub enum Pattern {
    Missing,
    Discard,
    Binding(EntityID),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Pattern]>),
    Variant(EntityID, usize, Option<Box<[Pattern]>>),
    Record(EntityID, Box<[Pattern]>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constructor {
    Missing,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(usize),
    Variant(EntityID, usize),
    Record(EntityID),
}

impl Pattern {
    pub fn is_exhaustive(&self) -> bool {
        match self {
            Self::Missing => false,
            Self::Discard => true,
            Self::Binding(_) => true,
            Self::Int(_) => false,
            Self::Float(_) => false,
            Self::String(_) => false,
            Self::Bool(_) => false,
            Self::Tuple(items) => items.iter().all(|item| item.is_exhaustive()),
            Self::Variant(_, _, _) => false,
            Self::Record(_, fields) => fields.iter().all(|field| field.is_exhaustive()),
        }
    }

    pub fn constructor(&self) -> Constructor {
        use Constructor as C;
        match self {
            Self::Missing => C::Missing,
            Self::Discard => C::Missing, // discards are not constructors
            Self::Binding(_) => C::Missing, // bindings are not constructors
            Self::Int(i) => C::Int(*i),
            Self::Float(f) => C::Float(*f),
            Self::String(s) => C::String(s.clone()),
            Self::Bool(b) => C::Bool(*b),
            Self::Tuple(items) => C::Tuple(items.len()),
            Self::Variant(id, var, _) => C::Variant(*id, *var),
            Self::Record(id, _) => C::Record(*id),
        }
    }

    pub fn constructor_args(&self) -> Vec<Pattern> {
        match self {
            Self::Missing => vec![],
            Self::Discard => vec![],
            Self::Binding(_) => vec![],
            Self::Int(_) => vec![],
            Self::Float(_) => vec![],
            Self::String(_) => vec![],
            Self::Bool(_) => vec![],
            Self::Tuple(items) => items.to_vec(),
            Self::Variant(_, _, None) => vec![],
            Self::Variant(_, _, Some(items)) => items.to_vec(),
            Self::Record(_, fields) => fields.to_vec(),
        }
    }

    fn collect_bindings(&self, bindings: &mut Vec<EntityID>) {
        match self {
            Self::Missing => {}
            Self::Int(_) => {}
            Self::Float(_) => {}
            Self::String(_) => {}
            Self::Bool(_) => {}
            Self::Discard => {}
            Self::Binding(id) => bindings.push(*id),
            Self::Tuple(items) => {
                for item in items {
                    item.collect_bindings(bindings);
                }
            }
            Self::Variant(_, _, None) => {}
            Self::Variant(_, _, Some(items)) => {
                for item in items {
                    item.collect_bindings(bindings);
                }
            }
            Self::Record(_, fields) => {
                for field in fields {
                    field.collect_bindings(bindings);
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

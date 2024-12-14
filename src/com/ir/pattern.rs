use super::EntityID;

#[derive(Debug, Clone)]
pub enum Pattern {
    Missing,
    Binding(EntityID),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Pattern]>),
}

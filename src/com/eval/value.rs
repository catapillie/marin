#[derive(Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Value]>),
    Array(Box<[Value]>),
}

pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Bundle(Box<[Value]>),
}

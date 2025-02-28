use std::fmt::Display;

pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Bundle(Box<[Value]>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(d) => write!(f, "{d}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Bundle(items) => {
                write!(f, "[")?;
                let mut iter = items.iter().peekable();
                while let Some(item) = iter.next() {
                    item.fmt(f)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
        }
    }
}

use std::fmt::Display;

#[derive(PartialEq)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Func,
    Bundle(Box<[Value]>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(d) => write!(f, "{d}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Func => write!(f, "<fun>"),
            Value::Bundle(items) => {
                write!(f, "(")?;
                let mut iter = items.iter().peekable();
                while let Some(item) = iter.next() {
                    item.fmt(f)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

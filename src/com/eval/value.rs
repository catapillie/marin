use crate::com::ir;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Tuple(Box<[Value]>),
    Array(Box<[Value]>),
    Lambda(Box<ir::Signature>, Box<ir::Expr>),
}

impl Value {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value as V;
        match self {
            V::Int(n) => write!(f, "{n}"),
            V::Float(d) => write!(f, "{d}"),
            V::String(s) => write!(f, "{s:?}"),
            V::Bool(b) => write!(f, "{b}"),
            V::Tuple(items) => {
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
            V::Array(items) => {
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
            V::Lambda(..) => write!(f, "<fun>"),
        }
    }
}

use crate::com::loc::Loc;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct TypeID(pub usize);

pub struct TypeNode {
    pub parent: TypeID,
    pub ty: Type,
    pub loc: Option<Loc>,
}

#[derive(Clone)]
pub enum Type {
    Var,
    Int,
    Float,
    Bool,
    String,
    Tuple(Box<[TypeID]>),
    Array(TypeID),
    Lambda(Box<[TypeID]>, TypeID),
}

impl Type {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

#[derive(Debug, Clone)]
pub enum TypeRepr {
    Name(String),
    Int,
    Float,
    Bool,
    String,
    Tuple(Box<[TypeRepr]>),
    Array(Box<TypeRepr>),
    Lambda(Box<[TypeRepr]>, Box<TypeRepr>),
}

impl TypeRepr {
    fn fmt_paren(&self, paren: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeRepr::Name(name) => write!(f, "{name}"),
            TypeRepr::Int => write!(f, "int"),
            TypeRepr::Float => write!(f, "float"),
            TypeRepr::Bool => write!(f, "bool"),
            TypeRepr::String => write!(f, "string"),
            TypeRepr::Tuple(items) => {
                write!(f, "(")?;
                let mut iter = items.iter().peekable();
                while let Some(item) = iter.next() {
                    item.fmt_paren(false, f)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            TypeRepr::Array(item) => {
                write!(f, "[]")?;
                item.fmt_paren(true, f)?;
                Ok(())
            }
            TypeRepr::Lambda(args, ret) => {
                if paren {
                    write!(f, "(")?;
                }
                let mut iter = args.iter().peekable();
                while let Some(item) = iter.next() {
                    item.fmt_paren(false, f)?;
                    match iter.peek() {
                        Some(_) => write!(f, ", ")?,
                        None => write!(f, " ")?,
                    }
                }
                write!(f, "-> ")?;
                ret.fmt_paren(false, f)?;
                if paren {
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for TypeRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_paren(false, f)
    }
}

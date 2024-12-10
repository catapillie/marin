use crate::com::{
    loc::Loc,
    reporting::{Label, Report},
};
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypeID(pub usize);

pub struct TypeNode {
    pub parent: TypeID,
    pub ty: Type,
    pub loc: Option<Loc>,
    pub provenances: Vec<TypeProvenance>,
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
pub enum TypeString {
    Name(String),
    Int,
    Float,
    Bool,
    String,
    Tuple(Box<[TypeString]>),
    Array(Box<TypeString>),
    Lambda(Box<[TypeString]>, Box<TypeString>),
}

impl TypeString {
    fn fmt_paren(&self, paren: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeString::Name(name) => write!(f, "{name}"),
            TypeString::Int => write!(f, "int"),
            TypeString::Float => write!(f, "float"),
            TypeString::Bool => write!(f, "bool"),
            TypeString::String => write!(f, "string"),
            TypeString::Tuple(items) => {
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
            TypeString::Array(item) => {
                write!(f, "[]")?;
                item.fmt_paren(true, f)?;
                Ok(())
            }
            TypeString::Lambda(args, ret) => {
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

impl Display for TypeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_paren(false, f)
    }
}

pub enum TypeProvenance {
    ReturnedFromBreak(Loc, Option<String>),
    NonExhaustiveConditional(crate::com::loc::Loc),
}

impl TypeProvenance {
    pub fn apply(&self, report: Report) -> Report {
        use TypeProvenance as Pr;
        match self {
            Pr::ReturnedFromBreak(loc, name) => {
                report.with_secondary_label(Label::ReturnedFromBreak(name.clone()), *loc)
            }
            Pr::NonExhaustiveConditional(loc) => {
                report.with_secondary_label(Label::NonExhaustiveConditionalUnit, *loc)
            }
        }
    }
}

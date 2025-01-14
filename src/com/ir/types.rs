use crate::com::{
    loc::Loc,
    reporting::{Label, Report},
};
use std::{collections::HashSet, fmt::Display};

use super::EntityID;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeID(pub usize);

pub struct TypeNode {
    pub parent: TypeID,
    pub ty: Type,
    pub loc: Option<Loc>,
    pub depth: usize,
    pub provenances: Vec<TypeProvenance>,
}

#[derive(Clone)]
pub struct Scheme {
    pub forall: HashSet<TypeID>,
    pub uninstantiated: TypeID,
}

impl Scheme {
    pub fn mono(ty: TypeID) -> Self {
        Self {
            forall: HashSet::new(),
            uninstantiated: ty,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Type {
    Var,
    Int,
    Float,
    Bool,
    String,
    Tuple(Box<[TypeID]>),
    Array(TypeID),
    Lambda(Box<[TypeID]>, TypeID),
    Record(EntityID, Option<Box<[TypeID]>>),
    Union(EntityID, Option<Box<[TypeID]>>),
}

impl Type {
    pub fn unit() -> Self {
        Self::Tuple(Box::new([]))
    }
}

#[derive(Debug, Clone)]
pub enum TypeString {
    Hidden,
    Name(String),
    Int,
    Float,
    Bool,
    String,
    Tuple(Box<[TypeString]>),
    Array(Box<TypeString>),
    Lambda(Box<[TypeString]>, Box<TypeString>),
    Constructor(String, Box<[TypeString]>),
}

impl TypeString {
    fn fmt_paren(&self, paren: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hidden => write!(f, "_"),
            Self::Name(name) => write!(f, "{name}"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "string"),
            Self::Tuple(items) => {
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
            Self::Array(item) => {
                write!(f, "[]")?;
                item.fmt_paren(true, f)?;
                Ok(())
            }
            Self::Lambda(args, ret) => {
                if paren {
                    write!(f, "(")?;
                }
                let mut iter = args.iter().peekable();
                while let Some(item) = iter.next() {
                    item.fmt_paren(true, f)?;
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
            Self::Constructor(name, items) => {
                write!(f, "{name}(")?;
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
        }
    }
}

impl Display for TypeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_paren(false, f)
    }
}

#[derive(Debug, Clone)]
pub struct SchemeString {
    pub forall: Box<[String]>,
    pub uninstantiated: TypeString,
}

impl Display for SchemeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.forall.is_empty() {
            return self.uninstantiated.fmt(f);
        }

        write!(f, "forall")?;
        for x in &self.forall {
            write!(f, " {x}")?;
        }
        write!(f, ", {}", self.uninstantiated)?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum TypeProvenance {
    ReturnedFromBreak(Loc, Option<String>),
    NonExhaustiveConditional(Loc),
    VariableDefinition(Loc, String),
    VariantDefinition(Loc, String, Loc, String),
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
            Pr::VariableDefinition(loc, name) => {
                report.with_secondary_label(Label::VariableDefinition(name.clone()), *loc)
            }
            Pr::VariantDefinition(variant_loc, variant_name, union_loc, union_name) => report
                .with_secondary_label(Label::VariantDefinition(variant_name.clone()), *variant_loc)
                .with_secondary_label(Label::UnionDefinition(union_name.clone()), *union_loc),
        }
    }
}

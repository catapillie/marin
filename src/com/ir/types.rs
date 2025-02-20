use colored::Colorize;

use super::EntityID;
use crate::com::{
    loc::Loc,
    reporting::{Label, Report},
};
use std::{collections::BTreeSet, fmt::Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub forall: BTreeSet<TypeID>,
    pub uninstantiated: TypeID,
    pub constraints: Vec<Constraint>,
}

impl Scheme {
    pub fn mono(ty: TypeID) -> Self {
        Self {
            forall: BTreeSet::new(),
            uninstantiated: ty,
            constraints: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct InstanceScheme {
    pub forall: BTreeSet<TypeID>,
    pub constraint: Constraint,
    pub required_constraints: Vec<Constraint>,
}

#[derive(Clone)]
pub struct Constraint {
    pub id: EntityID,
    pub loc: Loc,
    pub class_args: Box<[TypeID]>,
    pub associated_args: Box<[TypeID]>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            Self::Lambda(_, _) => {
                if paren {
                    write!(f, "(")?;
                }

                write!(f, "fun")?;
                let mut ty = self;
                while let Self::Lambda(args, ret) = ty {
                    write!(f, "(")?;
                    let mut iter = args.iter().peekable();
                    while let Some(item) = iter.next() {
                        item.fmt_paren(true, f)?;
                        if iter.peek().is_some() {
                            write!(f, ", ")?;
                        }
                    }
                    write!(f, ")")?;

                    ty = ret;
                }

                write!(f, " => ")?;
                ty.fmt_paren(false, f)?;
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
    pub constraints: Box<[ConstraintString]>,
}

#[derive(Debug, Clone)]
pub struct InstanceSchemeString {
    pub forall: Box<[String]>,
    pub constraint: ConstraintString,
    pub required_constraints: Box<[ConstraintString]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstraintString {
    pub name: String,
    pub class_args: Box<[TypeString]>,
    pub associated_args: Box<[TypeString]>,
}

impl Display for SchemeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.forall.is_empty() {
            return self.uninstantiated.fmt(f);
        }

        write!(f, "{}", "forall".bold())?;
        for x in &self.forall {
            write!(f, " {x}")?;
        }
        write!(f, ", {}", self.uninstantiated.to_string().underline())?;

        if self.constraints.is_empty() {
            return Ok(());
        }

        write!(f, ", {} ", "where".bold())?;
        let mut iter = self.constraints.iter().peekable();
        while let Some(constraint) = iter.next() {
            write!(f, "[{constraint}]")?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

impl Display for InstanceSchemeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.forall.is_empty() {
            write!(f, "{}", "forall".bold())?;
            for x in &self.forall {
                write!(f, " {x}")?;
            }
            write!(f, ", ")?;
        };

        write!(
            f,
            "{} [{}]",
            "have".bold(),
            self.constraint.to_string().underline()
        )?;

        if self.required_constraints.is_empty() {
            return Ok(());
        }

        write!(f, ", {} ", "where".bold())?;
        let mut iter = self.required_constraints.iter().peekable();
        while let Some(constraint) = iter.next() {
            write!(f, "[{constraint}]")?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

impl Display for ConstraintString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.name)?;
        let mut iter = self.class_args.iter().peekable();
        while let Some(arg) = iter.next() {
            write!(f, "{arg}")?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;

        if !self.associated_args.is_empty() {
            write!(f, " of")?;
            for arg in &self.associated_args {
                write!(f, " {arg}")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum TypeProvenance {
    ReturnedFromBreak(Loc, Option<String>),
    NonExhaustiveConditional(Loc),
    VariableDefinition(Loc, String),
    VariantDefinition(Loc, String, Loc, String),
    ClassItemDefinition(Loc, String, Loc, String),
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
            Pr::ClassItemDefinition(item_loc, item_name, class_loc, class_name) => report
                .with_secondary_label(Label::ClassDefinition(class_name.clone()), *class_loc)
                .with_secondary_label(Label::ClassItemDefinition(item_name.clone()), *item_loc),
        }
    }
}

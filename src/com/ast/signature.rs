use super::{Expr, Pattern};
use crate::com::loc::Span;

#[derive(Debug)]
pub enum Signature {
    Missing,
    Name(Span, Box<Signature>),
    Args(Box<[Pattern]>, Box<Signature>),
    Empty,
}

impl Signature {
    fn collect_arg_patterns<'a>(&'a self, patterns: &mut Vec<&'a Pattern>) {
        match self {
            Self::Missing => {}
            Self::Name(_, next) => next.collect_arg_patterns(patterns),
            Self::Args(args, next) => {
                for arg in args {
                    patterns.push(arg);
                }
                next.collect_arg_patterns(patterns);
            }
            Self::Empty => {}
        }
    }

    pub fn arg_patterns(&self) -> Vec<&Pattern> {
        let mut patterns = Vec::new();
        self.collect_arg_patterns(&mut patterns);
        patterns
    }
}

#[derive(Debug)]
pub enum TypeSignature {
    Missing,
    Args(Box<[Expr]>, Box<TypeSignature>),
    Empty,
}

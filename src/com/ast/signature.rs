use super::Pattern;
use crate::com::loc::Span;

#[derive(Debug)]
pub enum Signature {
    Missing,
    Name(Span, Box<Signature>),
    Args(Box<[Pattern]>, Box<Signature>),
    Empty,
}

use logos::Span;

use super::{mix_spans, Expr};

#[derive(Debug, Clone)]
pub enum Label {
    Named(NamedLabel),
    Empty(Span),
}

#[derive(Debug, Clone)]
pub struct NamedLabel {
    pub left_chev: Span,
    pub right_chev: Span,
    pub name_expr: Box<Expr>,
}

impl Label {
    pub fn span(&self) -> Span {
        match self {
            Label::Named(named) => mix_spans([
                named.left_chev.clone(),
                named.right_chev.clone(),
                named.name_expr.span(),
            ]),
            Label::Empty(span) => span.clone(),
        }
    }
}

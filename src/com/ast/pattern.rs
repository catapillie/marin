use super::mix_spans;
use crate::com::loc::Span;

#[derive(Debug)]
pub enum Pattern {
    Missing(Span),
    Binding(Span),
    Int(Span),
    Float(Span),
    String(Span),
    True(Span),
    False(Span),
    Tuple(Span, Span, Box<[Pattern]>),
}

impl Pattern {
    pub fn span(&self) -> Span {
        use Pattern as P;
        match self {
            P::Missing(span) => *span,
            P::Binding(span) => *span,
            P::Int(span) => *span,
            P::Float(span) => *span,
            P::String(span) => *span,
            P::True(span) => *span,
            P::False(span) => *span,
            P::Tuple(left_paren, right_paren, items) => {
                mix_spans([*left_paren, *right_paren, item_spans(items)])
            }
        }
    }

    pub fn is_irrefutable(&self) -> bool {
        use Pattern as P;
        match self {
            P::Missing(_) => true,
            P::Binding(_) => true,
            P::Int(_) => false,
            P::Float(_) => false,
            P::String(_) => false,
            P::True(_) => false,
            P::False(_) => false,
            P::Tuple(_, _, items) => items.iter().all(Self::is_irrefutable),
        }
    }
}

fn item_spans(items: &[Pattern]) -> Span {
    mix_spans(items.iter().map(|e| e.span()))
}

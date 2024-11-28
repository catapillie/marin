mod expr;
mod label;
mod traversal;

pub use expr::*;
pub use label::*;
pub use traversal::*;

pub struct File(pub Box<[Expr]>);

use super::loc::Span;

fn item_spans(items: &[Expr]) -> Span {
    mix_spans(items.iter().map(|e| e.span()))
}

fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans.into_iter().fold(Span::default(), Span::combine)
}

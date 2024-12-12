mod expr;
mod label;
mod traversal;
mod pattern;

pub use expr::*;
pub use label::*;
pub use traversal::*;
pub use pattern::*;

pub struct File(pub Box<[Expr]>);

use super::loc::Span;

fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans.into_iter().fold(Span::default(), Span::combine)
}

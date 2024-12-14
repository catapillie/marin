mod expr;
mod label;
mod pattern;
mod signature;
mod traversal;

pub use expr::*;
pub use label::*;
pub use pattern::*;
pub use signature::*;
pub use traversal::*;

pub struct File(pub Box<[Expr]>);

use super::loc::Span;

fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans.into_iter().fold(Span::default(), Span::combine)
}

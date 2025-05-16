mod bin_op;
mod expr;
mod label;
mod pattern;
mod signature;
mod traversal;
mod un_op;

pub use bin_op::*;
pub use expr::*;
pub use label::*;
pub use pattern::*;
pub use signature::*;
pub use traversal::*;
pub use un_op::*;

pub struct File(pub Box<[Expr]>);

use super::loc::Span;

fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans.into_iter().fold(Span::default(), Span::combine)
}

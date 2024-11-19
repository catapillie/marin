mod expr;
mod label;

pub use expr::*;
pub use label::*;
use logos::Span;

#[allow(clippy::reversed_empty_ranges)]
const EMPTY_SPAN: Span = usize::MAX..usize::MIN;

fn item_spans(items: &[Expr]) -> Span {
    mix_spans(items.iter().map(|e| e.span()))
}

fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans
        .into_iter()
        .fold(EMPTY_SPAN, |left, right| {
            usize::min(left.start, right.start)..usize::max(left.end, right.end)
        })
}

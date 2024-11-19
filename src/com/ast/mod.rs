mod expr;
mod label;

pub use expr::*;
pub use label::*;
use logos::Span;

fn item_spans(items: &[Expr]) -> Span {
    mix_spans(items.iter().map(|e| e.span()))
}

#[allow(clippy::reversed_empty_ranges)]
fn mix_spans(spans: impl IntoIterator<Item = Span>) -> Span {
    spans
        .into_iter()
        .fold(usize::MAX..usize::MIN, |left, right| {
            usize::min(left.start, right.start)..usize::max(left.end, right.end)
        })
}

use logos::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Missing(Lexeme),
    Int(Lexeme),
    Float(Lexeme),
    String(Lexeme),
    True(Lexeme),
    False(Lexeme),
    Tuple(Tuple),
    Array(Array),
}

#[derive(Debug, Clone)]
pub struct Lexeme {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Tuple {
    pub left_paren: Span,
    pub right_paren: Span,
    pub items: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct Array {
    pub left_bracket: Span,
    pub right_bracket: Span,
    pub items: Box<[Expr]>,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Missing(lex) => lex.span.clone(),
            Expr::Int(lex) => lex.span.clone(),
            Expr::Float(lex) => lex.span.clone(),
            Expr::String(lex) => lex.span.clone(),
            Expr::True(lex) => lex.span.clone(),
            Expr::False(lex) => lex.span.clone(),
            Expr::Tuple(tuple) => mix_spans([
                tuple.left_paren.clone(),
                item_spans(&tuple.items),
                tuple.right_paren.clone(),
            ]),
            Expr::Array(array) => mix_spans([
                array.left_bracket.clone(),
                item_spans(&array.items),
                array.right_bracket.clone(),
            ]),
        }
    }
}

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

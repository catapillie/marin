use super::{item_spans, mix_spans, Label};
use crate::com::loc::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Missing(Lexeme),
    Int(Lexeme),
    Float(Lexeme),
    String(Lexeme),
    True(Lexeme),
    False(Lexeme),
    Var(Lexeme),
    Tuple(Tuple),
    Array(Array),
    Spread(Spread),
    Block(Block),
    Loop(Loop),
    Conditional(Conditional),
    Break(Break),
    Skip(Skip),
    Call(Call),
    Access(Access),
    Let(Let),
    Fun(Fun),
    Import(Import),
    Super(Lexeme),
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

#[derive(Debug, Clone)]
pub struct Spread {
    pub spread: Span,
    pub name: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub do_kw: Span,
    pub end_kw: Span,
    pub label: Box<Label>,
    pub items: Box<[Expr]>,
}
#[derive(Debug, Clone)]

pub struct Loop {
    pub loop_kw: Span,
    pub end_kw: Span,
    pub label: Box<Label>,
    pub items: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub first_branch: Box<Branch>,
    pub else_branches: Box<[(Span, Branch)]>,
    pub end_kw: Span,
}

#[derive(Debug, Clone)]
pub enum Branch {
    If(IfBranch),
    While(WhileBranch),
    Match(MatchBranch),
    Fallback(FallbackBranch),
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub if_kw: Span,
    pub then_kw: Span,
    pub label: Box<Label>,
    pub guard: Box<Expr>,
    pub body: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct WhileBranch {
    pub while_kw: Span,
    pub do_kw: Span,
    pub label: Box<Label>,
    pub guard: Box<Expr>,
    pub body: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub match_kw: Span,
    pub with_kw: Span,
    pub label: Box<Label>,
    pub scrutinee: Box<Expr>,
    pub cases: Box<[MatchCase]>,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub maps: Span,
    pub pattern: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct FallbackBranch {
    pub label: Box<Label>,
    pub body: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct Break {
    pub break_kw: Span,
    pub label: Box<Label>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Skip {
    pub skip_kw: Span,
    pub label: Box<Label>,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub left_paren: Span,
    pub right_paren: Span,
    pub callee: Box<Expr>,
    pub args: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct Access {
    pub dot: Span,
    pub name: Span,
    pub accessed: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub let_kw: Span,
    pub assign: Option<Span>,
    pub pattern: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Fun {
    pub fun_kw: Span,
    pub maps: Option<Span>,
    pub signature: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub import_kw: Span,
    pub queries: Box<[Expr]>,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Missing(e) => e.span,
            Expr::Int(e) => e.span,
            Expr::Float(e) => e.span,
            Expr::String(e) => e.span,
            Expr::True(e) => e.span,
            Expr::False(e) => e.span,
            Expr::Var(e) => e.span,
            Expr::Tuple(e) => mix_spans([e.left_paren, item_spans(&e.items), e.right_paren]),
            Expr::Array(e) => mix_spans([e.left_bracket, item_spans(&e.items), e.right_bracket]),
            Expr::Spread(e) => mix_spans([e.spread, e.name.unwrap_or(Span::default())]),
            Expr::Block(e) => mix_spans([e.do_kw, e.label.span(), item_spans(&e.items), e.end_kw]),
            Expr::Loop(e) => mix_spans([e.loop_kw, e.label.span(), item_spans(&e.items), e.end_kw]),
            Expr::Conditional(e) => mix_spans([
                e.first_branch.span(),
                mix_spans(
                    e.else_branches
                        .iter()
                        .map(|(else_kw, branch)| mix_spans([*else_kw, branch.span()])),
                ),
                e.end_kw,
            ]),
            Expr::Break(e) => mix_spans([
                e.break_kw,
                e.label.span(),
                e.expr.as_ref().map(|e| e.span()).unwrap_or(Span::default()),
            ]),
            Expr::Skip(e) => mix_spans([e.skip_kw, e.label.span()]),
            Expr::Call(e) => mix_spans([
                e.callee.span(),
                e.left_paren,
                item_spans(&e.args),
                e.right_paren,
            ]),
            Expr::Access(e) => mix_spans([e.accessed.span(), e.dot, e.name]),
            Expr::Let(e) => mix_spans([
                e.let_kw,
                e.pattern.span(),
                e.assign.unwrap_or(Span::default()),
                e.value.span(),
            ]),
            Expr::Fun(e) => mix_spans([
                e.fun_kw,
                e.signature.span(),
                e.maps.unwrap_or(Span::default()),
                e.value.span(),
            ]),
            Expr::Import(e) => mix_spans([e.import_kw, item_spans(&e.queries)]),
            Expr::Super(e) => e.span,
        }
    }
}

impl Branch {
    pub fn span(&self) -> Span {
        match self {
            Branch::If(b) => mix_spans([
                b.if_kw,
                b.label.span(),
                b.guard.span(),
                b.then_kw,
                item_spans(&b.body),
            ]),
            Branch::While(b) => mix_spans([
                b.while_kw,
                b.label.span(),
                b.guard.span(),
                b.do_kw,
                item_spans(&b.body),
            ]),
            Branch::Match(b) => {
                mix_spans([
                    b.match_kw,
                    b.label.span(),
                    b.scrutinee.span(),
                    b.with_kw,
                    mix_spans(b.cases.iter().map(|case| {
                        mix_spans([case.pattern.span(), case.maps, case.value.span()])
                    })),
                ])
            }
            Branch::Fallback(b) => mix_spans([b.label.span(), item_spans(&b.body)]),
        }
    }
}

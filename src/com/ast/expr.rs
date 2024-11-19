use logos::Span;

use super::{item_spans, mix_spans, Label, EMPTY_SPAN};

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
    Block(Block),
    Conditional(Conditional),
    Break(Break),
    Skip(Skip),
    Call(Call),
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
pub struct Block {
    pub do_kw: Span,
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

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Missing(e) => e.span.clone(),
            Expr::Int(e) => e.span.clone(),
            Expr::Float(e) => e.span.clone(),
            Expr::String(e) => e.span.clone(),
            Expr::True(e) => e.span.clone(),
            Expr::False(e) => e.span.clone(),
            Expr::Var(e) => e.span.clone(),
            Expr::Tuple(e) => mix_spans([
                e.left_paren.clone(),
                item_spans(&e.items),
                e.right_paren.clone(),
            ]),
            Expr::Array(e) => mix_spans([
                e.left_bracket.clone(),
                item_spans(&e.items),
                e.right_bracket.clone(),
            ]),
            Expr::Block(e) => mix_spans([
                e.do_kw.clone(),
                e.label.span(),
                item_spans(&e.items),
                e.end_kw.clone(),
            ]),
            Expr::Conditional(e) => mix_spans([
                e.first_branch.span(),
                mix_spans(
                    e.else_branches
                        .iter()
                        .map(|(else_kw, branch)| mix_spans([else_kw.clone(), branch.span()])),
                ),
                e.end_kw.clone(),
            ]),
            Expr::Break(e) => mix_spans([
                e.break_kw.clone(),
                e.label.span(),
                e.expr.as_ref().map(|e| e.span()).unwrap_or(EMPTY_SPAN),
            ]),
            Expr::Skip(e) => mix_spans([e.skip_kw.clone(), e.label.span()]),
            Expr::Call(e) => mix_spans([
                e.callee.span(),
                e.left_paren.clone(),
                item_spans(&e.args),
                e.right_paren.clone(),
            ]),
        }
    }
}

impl Branch {
    pub fn span(&self) -> Span {
        match self {
            Branch::If(b) => mix_spans([
                b.if_kw.clone(),
                b.label.span(),
                b.guard.span(),
                b.then_kw.clone(),
                item_spans(&b.body),
            ]),
            Branch::While(b) => mix_spans([
                b.while_kw.clone(),
                b.label.span(),
                b.guard.span(),
                b.do_kw.clone(),
                item_spans(&b.body),
            ]),
            Branch::Fallback(b) => mix_spans([b.label.span(), item_spans(&b.body)]),
        }
    }
}

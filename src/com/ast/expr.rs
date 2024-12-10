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
pub struct Conditional {
    pub first_branch: Box<Branch>,
    pub else_branches: Box<[(Span, Branch)]>,
    pub end_kw: Span,
}

#[derive(Debug, Clone)]
pub enum Branch {
    If(IfBranch),
    While(WhileBranch),
    Loop(LoopBranch),
    Match(MatchBranch),
    Else(ElseBranch),
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub if_kw: Span,
    pub then_kw: Span,
    pub label: Box<Label>,
    pub condition: Box<Expr>,
    pub body: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct WhileBranch {
    pub while_kw: Span,
    pub do_kw: Span,
    pub label: Box<Label>,
    pub condition: Box<Expr>,
    pub body: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct LoopBranch {
    pub loop_kw: Span,
    pub label: Box<Label>,
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
pub struct ElseBranch {
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
    pub accessed: Box<Expr>,
    pub accessor: Box<Expr>,
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

impl Tuple {
    pub fn span(&self) -> Span {
        mix_spans([self.left_paren, item_spans(&self.items), self.right_paren])
    }
}

impl Array {
    pub fn span(&self) -> Span {
        mix_spans([
            self.left_bracket,
            item_spans(&self.items),
            self.right_bracket,
        ])
    }
}

impl Spread {
    pub fn span(&self) -> Span {
        mix_spans([self.spread, self.name.unwrap_or_default()])
    }
}

impl Block {
    pub fn span(&self) -> Span {
        mix_spans([
            self.do_kw,
            self.label.span(),
            item_spans(&self.items),
            self.end_kw,
        ])
    }
}

impl Conditional {
    pub fn span(&self) -> Span {
        mix_spans([
            self.first_branch.span(),
            mix_spans(
                self.else_branches
                    .iter()
                    .map(|(else_kw, branch)| mix_spans([*else_kw, branch.span()])),
            ),
            self.end_kw,
        ])
    }
}

impl Break {
    pub fn span(&self) -> Span {
        mix_spans([
            self.break_kw,
            self.label.span(),
            self.expr.as_ref().map(|e| e.span()).unwrap_or_default(),
        ])
    }
}

impl Skip {
    pub fn span(&self) -> Span {
        mix_spans([self.skip_kw, self.label.span()])
    }
}

impl Call {
    pub fn span(&self) -> Span {
        mix_spans([
            self.callee.span(),
            self.left_paren,
            item_spans(&self.args),
            self.right_paren,
        ])
    }
}

impl Access {
    pub fn span(&self) -> Span {
        mix_spans([self.accessed.span(), self.dot, self.accessor.span()])
    }
}

impl Let {
    pub fn span(&self) -> Span {
        mix_spans([
            self.let_kw,
            self.pattern.span(),
            self.assign.unwrap_or_default(),
            self.value.span(),
        ])
    }
}

impl Fun {
    pub fn span(&self) -> Span {
        mix_spans([
            self.fun_kw,
            self.signature.span(),
            self.maps.unwrap_or_default(),
            self.value.span(),
        ])
    }
}

impl Import {
    pub fn span(&self) -> Span {
        mix_spans([self.import_kw, item_spans(&self.queries)])
    }
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
            Expr::Tuple(e) => e.span(),
            Expr::Array(e) => e.span(),
            Expr::Spread(e) => e.span(),
            Expr::Block(e) => e.span(),
            Expr::Conditional(e) => e.span(),
            Expr::Break(e) => e.span(),
            Expr::Skip(e) => e.span(),
            Expr::Call(e) => e.span(),
            Expr::Access(e) => e.span(),
            Expr::Let(e) => e.span(),
            Expr::Fun(e) => e.span(),
            Expr::Import(e) => e.span(),
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
                b.condition.span(),
                b.then_kw,
                item_spans(&b.body),
            ]),
            Branch::While(b) => mix_spans([
                b.while_kw,
                b.label.span(),
                b.condition.span(),
                b.do_kw,
                item_spans(&b.body),
            ]),
            Branch::Loop(b) => mix_spans([b.loop_kw, b.label.span(), item_spans(&b.body)]),
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
            Branch::Else(b) => mix_spans([b.label.span(), item_spans(&b.body)]),
        }
    }
}

use super::{BinOp, Label, UnOp, mix_spans};
use crate::com::loc::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Missing(Lexeme),
    Int(Lexeme),
    Float(Lexeme),
    String(Lexeme),
    Builtin(Lexeme),
    True(Lexeme),
    False(Lexeme),
    Underscores(Lexeme),
    Var(Lexeme),
    Tuple(Tuple),
    Array(Array),
    Block(Block),
    Conditional(Conditional),
    Break(Break),
    Skip(Skip),
    Call(Call),
    Index(Index),
    Access(Access),
    Let(Let),
    Pub(Pub),
    Fun(Fun),
    Alias(Alias),
    Import(Import),
    ImportFrom(ImportFrom),
    Super(Lexeme),
    Record(Record),
    RecordValue(RecordValue),
    Union(Union),
    Class(Class),
    Have(Have),
    Binary(Binary),
    Unary(Unary),
    ArrayType(ArrayType),
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
pub struct Index {
    pub left_bracket: Span,
    pub right_bracket: Span,
    pub indexed: Box<Expr>,
    pub indices: Box<[Expr]>,
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
pub struct Pub {
    pub pub_kw: Span,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Fun {
    pub fun_kw: Span,
    pub maps: Option<Span>,
    pub signature: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub alias_kw: Span,
    pub as_kw: Span,
    pub path: Box<Expr>,
    pub name: Span,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub import_kw: Span,
    pub queries: Box<[ImportQuery]>,
}

#[derive(Debug, Clone)]
pub struct ImportFrom {
    pub import_kw: Span,
    pub from_kw: Span,
    pub queries: Box<[ImportQuery]>,
    pub path_query: Box<Expr>,
    pub path_query_uid: usize,
}

#[derive(Debug, Clone)]
pub struct ImportQuery {
    pub uid: usize,
    pub query: Box<Expr>,
    pub alias: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub record_kw: Span,
    pub end_kw: Span,
    pub signature: Box<Expr>,
    pub fields: Box<[(Expr, Expr)]>,
}

#[derive(Debug, Clone)]
pub struct RecordValue {
    pub left_brace: Span,
    pub right_brace: Span,
    pub fields: Box<[(Expr, Option<Expr>)]>,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub union_kw: Span,
    pub end_kw: Span,
    pub signature: Box<Expr>,
    pub variants: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub enum ClassItem {
    Constant,
    Function,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub class_kw: Span,
    pub end_kw: Span,
    pub signature: Box<Expr>,
    pub associated: Option<Box<[Expr]>>,
    pub items: Box<[(ClassItem, Expr, Expr)]>,
}

#[derive(Debug, Clone)]
pub struct Have {
    pub have_kw: Span,
    pub end_kw: Span,
    pub class: Box<Expr>,
    pub items: Box<[Expr]>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: BinOp,
    pub op_tok: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: UnOp,
    pub op_tok: Span,
    pub arg: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub left_bracket: Span,
    pub right_bracket: Span,
    pub ty: Box<Expr>,
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

impl Index {
    pub fn span(&self) -> Span {
        mix_spans([
            self.indexed.span(),
            self.left_bracket,
            item_spans(&self.indices),
            self.right_bracket,
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

impl Pub {
    pub fn span(&self) -> Span {
        mix_spans([self.pub_kw, self.expr.span()])
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

impl Alias {
    pub fn span(&self) -> Span {
        mix_spans([self.alias_kw, self.path.span(), self.as_kw, self.name])
    }
}

impl Import {
    pub fn span(&self) -> Span {
        mix_spans([
            self.import_kw,
            mix_spans(self.queries.iter().map(|q| match q.alias {
                Some(alias) => mix_spans([q.query.span(), alias]),
                None => q.query.span(),
            })),
        ])
    }
}

impl ImportFrom {
    pub fn span(&self) -> Span {
        mix_spans([
            self.import_kw,
            mix_spans(self.queries.iter().map(|q| match q.alias {
                Some(alias) => mix_spans([q.query.span(), alias]),
                None => q.query.span(),
            })),
            self.from_kw,
            self.path_query.span(),
        ])
    }
}

impl Record {
    pub fn span(&self) -> Span {
        mix_spans([
            self.record_kw,
            self.signature.span(),
            mix_spans(
                self.fields
                    .iter()
                    .map(|(expr, ty)| mix_spans([expr.span(), ty.span()])),
            ),
            self.end_kw,
        ])
    }
}

impl RecordValue {
    pub fn span(&self) -> Span {
        mix_spans([
            self.left_brace,
            mix_spans(self.fields.iter().map(|(expr, _)| expr.span())),
            self.right_brace,
        ])
    }
}

impl Union {
    pub fn span(&self) -> Span {
        mix_spans([
            self.union_kw,
            self.signature.span(),
            item_spans(&self.variants),
            self.end_kw,
        ])
    }
}

impl Class {
    pub fn span(&self) -> Span {
        mix_spans([
            self.class_kw,
            self.signature.span(),
            mix_spans(
                self.items
                    .iter()
                    .map(|(_, sig, ty)| mix_spans([sig.span(), ty.span()])),
            ),
            self.end_kw,
        ])
    }
}

impl Have {
    pub fn span(&self) -> Span {
        mix_spans([
            self.have_kw,
            self.class.span(),
            item_spans(&self.items),
            self.end_kw,
        ])
    }
}

impl Binary {
    pub fn span(&self) -> Span {
        mix_spans([self.left.span(), self.op_tok, self.right.span()])
    }
}

impl Unary {
    pub fn span(&self) -> Span {
        mix_spans([self.op_tok, self.arg.span()])
    }
}

impl ArrayType {
    pub fn span(&self) -> Span {
        mix_spans([self.left_bracket, self.right_bracket, self.ty.span()])
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Missing(e) => e.span,
            Self::Int(e) => e.span,
            Self::Float(e) => e.span,
            Self::String(e) => e.span,
            Self::Builtin(e) => e.span,
            Self::True(e) => e.span,
            Self::False(e) => e.span,
            Self::Underscores(e) => e.span,
            Self::Var(e) => e.span,
            Self::Tuple(e) => e.span(),
            Self::Array(e) => e.span(),
            Self::Block(e) => e.span(),
            Self::Conditional(e) => e.span(),
            Self::Break(e) => e.span(),
            Self::Skip(e) => e.span(),
            Self::Call(e) => e.span(),
            Self::Index(e) => e.span(),
            Self::Access(e) => e.span(),
            Self::Let(e) => e.span(),
            Self::Pub(e) => e.span(),
            Self::Fun(e) => e.span(),
            Self::Alias(e) => e.span(),
            Self::Import(e) => e.span(),
            Self::ImportFrom(e) => e.span(),
            Self::Super(e) => e.span,
            Self::Record(e) => e.span(),
            Self::RecordValue(e) => e.span(),
            Self::Union(e) => e.span(),
            Self::Class(e) => e.span(),
            Self::Have(e) => e.span(),
            Self::Binary(e) => e.span(),
            Self::Unary(e) => e.span(),
            Self::ArrayType(e) => e.span(),
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

fn item_spans(items: &[Expr]) -> Span {
    mix_spans(items.iter().map(|e| e.span()))
}

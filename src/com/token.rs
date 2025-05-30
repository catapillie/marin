use logos::Logos;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Logos, PartialEq, Eq)]
#[logos(skip r"[^\S\n\r]+|--[^\n\r]*")]
pub enum Token {
    Eof,
    #[regex(r"[\n\r](\s|--[^\n\r]*)*")]
    Newline,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("<")]
    LeftChev,
    #[token(">")]
    RightChev,

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("^")]
    BitXor,
    #[token("~")]
    BitNeg,

    #[token("=>")]
    Maps,
    #[token("=")]
    Assign,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("..")]
    Spread,
    #[token(".")]
    Dot,

    #[token("do")]
    Do,
    #[token("end")]
    End,
    #[token("let")]
    Let,
    #[token("pub")]
    Pub,
    #[token("fun")]
    Fun,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("loop")]
    Loop,
    #[token("match")]
    Match,
    #[token("with")]
    With,
    #[token("break")]
    Break,
    #[token("skip")]
    Skip,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("from")]
    From,
    #[token("alias")]
    Alias,
    #[token("super")]
    Super,
    #[token("record")]
    Record,
    #[token("union")]
    Union,
    #[token("class")]
    Class,
    #[token("of")]
    Of,
    #[token("have")]
    Have,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("xor")]
    Xor,
    #[token("not")]
    Not,

    #[regex(r"_+")]
    Underscores,
    #[regex(r"[^\W\d_]\w*")]
    Ident,
    #[regex(r"\d+")]
    Int,
    #[regex(r"\d+\.\d+")]
    Float,
    #[regex(r#""[^"]*""#)]
    String,
    #[regex(r"@[^\W]+")]
    Builtin,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Eof => write!(f, "end-of-file"),
            Token::Newline => write!(f, "newline"),

            Token::LeftParen => write!(f, "'('"),
            Token::RightParen => write!(f, "')'"),
            Token::LeftBracket => write!(f, "'['"),
            Token::RightBracket => write!(f, "']'"),
            Token::LeftBrace => write!(f, "'{{'"),
            Token::RightBrace => write!(f, "'}}'"),
            Token::LeftChev => write!(f, "'<'"),
            Token::RightChev => write!(f, "'>'"),

            Token::Add => write!(f, "'+'"),
            Token::Sub => write!(f, "'-'"),
            Token::Mul => write!(f, "'*'"),
            Token::Div => write!(f, "'/'"),
            Token::Mod => write!(f, "'%'"),
            Token::Eq => write!(f, "'=='"),
            Token::Ne => write!(f, "'!='"),
            Token::Le => write!(f, "'<='"),
            Token::Ge => write!(f, "'>='"),
            Token::BitAnd => write!(f, "'&'"),
            Token::BitOr => write!(f, "'|'"),
            Token::BitXor => write!(f, "'^'"),
            Token::BitNeg => write!(f, "'~'"),

            Token::Maps => write!(f, "'=>'"),
            Token::Assign => write!(f, "'='"),
            Token::Colon => write!(f, "':'"),
            Token::Comma => write!(f, "','"),
            Token::Spread => write!(f, "'..'"),
            Token::Dot => write!(f, "'.'"),

            Token::Do => write!(f, "'do' keyword"),
            Token::End => write!(f, "'end' keyword"),
            Token::Let => write!(f, "'let' keyword"),
            Token::Pub => write!(f, "'pub' keyword"),
            Token::Fun => write!(f, "'fun' keyword"),
            Token::If => write!(f, "'if' keyword"),
            Token::Then => write!(f, "'then' keyword"),
            Token::Else => write!(f, "'else' keyword"),
            Token::While => write!(f, "'while' keyword"),
            Token::Loop => write!(f, "'loop' keyword"),
            Token::Match => write!(f, "'match' keyword"),
            Token::With => write!(f, "'with' keyword"),
            Token::Break => write!(f, "'break' keyword"),
            Token::Skip => write!(f, "'skip' keyword"),
            Token::True => write!(f, "'true' keyword"),
            Token::False => write!(f, "'false' keyword"),
            Token::Import => write!(f, "'import' keyword"),
            Token::As => write!(f, "'as' keyword"),
            Token::From => write!(f, "'From' keyword"),
            Token::Alias => write!(f, "'alias' keyword"),
            Token::Super => write!(f, "'super' keyword"),
            Token::Record => write!(f, "'record' keyword"),
            Token::Union => write!(f, "'union' keyword"),
            Token::Class => write!(f, "'class' keyword"),
            Token::Of => write!(f, "'of' keyword"),
            Token::Have => write!(f, "'have' keyword"),
            Token::And => write!(f, "'and' keyword"),
            Token::Or => write!(f, "'or' keyword"),
            Token::Xor => write!(f, "'xor' keyword"),
            Token::Not => write!(f, "'not' keyword"),

            Token::Underscores => write!(f, "underscores"),
            Token::Ident => write!(f, "identifier"),
            Token::Int => write!(f, "integer literal"),
            Token::Float => write!(f, "floating-point literal"),
            Token::String => write!(f, "string literal"),
            Token::Builtin => write!(f, "built-in literal"),
        }
    }
}

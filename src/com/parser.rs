use logos::{Logos, SpannedIter};
use std::iter::Peekable;

use super::{
    ast::{self},
    loc::{Loc, Span},
    reporting::{Header, Label, Report},
    Token,
};

pub struct Parser<'src, 'e> {
    source: &'src str,
    file: usize,
    lexer: Peekable<SpannedIter<'src, Token>>,
    prev: Token,
    bounds: (usize, usize),
    reports: &'e mut Vec<Report>,
}

impl<'src, 'e> Parser<'src, 'e> {
    pub fn new(source: &'src str, file: usize, reports: &'e mut Vec<Report>) -> Self {
        let mut p = Parser {
            source,
            file,
            lexer: Token::lexer(source).spanned().peekable(),
            prev: Token::Eof,
            bounds: (0, 0),
            reports,
        };

        p.peek();
        p
    }

    fn pos_from(&self) -> usize {
        self.bounds.1
    }

    fn pos_to(&self) -> usize {
        self.bounds.0
    }

    fn span_here(&self) -> Span {
        Span::at(self.pos_from())
    }

    fn loc_here(&self) -> Loc {
        self.span_here().wrap(self.file)
    }

    fn loc_from_here(&self) -> Loc {
        Span::at(self.pos_to()).wrap(self.file)
    }

    fn peek(&mut self) -> Token {
        let (tok, start) = match self.lexer.peek() {
            Some((tok, span)) => (tok.ok(), span.start),
            None => (Some(Token::Eof), self.source.len()),
        };

        self.bounds.1 = start;
        if let Some(tok) = tok {
            return tok;
        }

        let mut span = start..start;
        while let Some((Err(()), tok_span)) = self.lexer.peek() {
            span.end = tok_span.end;
            self.lexer.next();
        }

        self.reports.push(
            Report::error(Header::InvalidCharacterSequence(
                self.source[span.clone()].to_string(),
            ))
            .with_primary_label(Label::Empty, Loc::new(span.into(), self.file)),
        );

        let (tok, start) = match self.lexer.peek() {
            Some((Ok(tok), span)) => (*tok, span.start),
            None => (Token::Eof, self.source.len()),
            _ => unreachable!("errors should have been skipped over"),
        };

        self.bounds.1 = start;
        tok
    }

    fn consume_token(&mut self) -> Span {
        let len = self.source.len();
        let ((tok, tok_span), start) = match self.lexer.next() {
            Some((tok, span)) => ((tok.unwrap(), span.clone()), span.end),
            None => ((Token::Eof, len..len), len),
        };

        self.prev = tok;
        self.bounds.0 = start;
        self.peek();
        tok_span.into()
    }

    fn try_expect_token(&mut self, expected: Token) -> Option<Span> {
        if self.peek() == expected {
            Some(self.consume_token())
        } else {
            None
        }
    }

    fn expect_token(&mut self, expected: Token) -> Span {
        let p = self.peek();
        if p == expected {
            self.consume_token()
        } else {
            self.reports.push(
                Report::error(Header::ExpectedToken(expected, p))
                    .with_primary_label(Label::Empty, self.loc_from_here()),
            );
            self.span_here()
        }
    }

    fn skip_newlines(&mut self) -> bool {
        self.prev == Token::Newline || self.try_expect_token(Token::Newline).is_some()
    }

    pub fn parse_file(&mut self) -> ast::File {
        let exprs = self.parse_newline_separated_items();
        self.expect_token(Token::Eof);

        ast::File(exprs)
    }

    pub fn expect_expression(&mut self) -> ast::Expr {
        match self.try_parse_expression() {
            Some(expr) => expr,
            None => {
                self.reports.push(
                    Report::error(Header::ExpectedExpression())
                        .with_primary_label(Label::Empty, self.loc_here()),
                );
                ast::Expr::Missing(ast::Lexeme {
                    span: self.span_here(),
                })
            }
        }
    }

    pub fn try_parse_expression(&mut self) -> Option<ast::Expr> {
        self.try_parse_primary_expression()
    }

    pub fn expect_primary_expression(&mut self) -> ast::Expr {
        match self.try_parse_primary_expression() {
            Some(expr) => expr,
            None => {
                self.reports.push(
                    Report::error(Header::ExpectedExpression())
                        .with_primary_label(Label::Empty, self.loc_here()),
                );
                ast::Expr::Missing(ast::Lexeme {
                    span: self.span_here(),
                })
            }
        }
    }

    pub fn try_parse_primary_expression(&mut self) -> Option<ast::Expr> {
        let mut expr = match self.peek() {
            Token::Int => self.try_parse_int_expression(),
            Token::Float => self.try_parse_float_expression(),
            Token::String => self.try_parse_string_expression(),
            Token::True => self.try_parse_true_expression(),
            Token::False => self.try_parse_false_expression(),
            Token::Ident => self.try_parse_var_expression(),
            Token::Spread => self.try_parse_spread_expression(),

            Token::LeftParen => self.try_parse_tuple_expression(),
            Token::LeftBracket => self.try_parse_array_expression(),

            Token::Do => self.try_parse_block_expression(),
            Token::If | Token::While | Token::Loop | Token::Match => {
                self.try_parse_conditional_expression()
            }
            Token::Break => self.try_parse_break_expression(),
            Token::Skip => self.try_parse_skip_expression(),
            Token::Fun => self.try_parse_fun_expression(),
            Token::Let => return self.try_parse_let_expression(),

            Token::Import => return self.try_parse_import_expression(),
            Token::Super => self.try_parse_super_expression(),
            _ => None,
        }?;

        loop {
            if let Some(left_paren) = self.try_expect_token(Token::LeftParen) {
                let args = self.parse_comma_separated_items();
                let right_paren = self.expect_token(Token::RightParen);

                expr = ast::Expr::Call(ast::Call {
                    left_paren,
                    right_paren,
                    callee: Box::new(expr),
                    args,
                });
                continue;
            }

            self.skip_newlines();
            if let Some(dot) = self.try_expect_token(Token::Dot) {
                let accessor = self.expect_accessor();
                expr = ast::Expr::Access(ast::Access {
                    dot,
                    accessor: Box::new(accessor),
                    accessed: Box::new(expr),
                });
                continue;
            }

            break;
        }

        Some(expr)
    }

    fn expect_accessor(&mut self) -> ast::Expr {
        let accessor = match self.peek() {
            Token::Ident => self.try_parse_var_expression(),
            Token::Super => self.try_parse_super_expression(),
            _ => None,
        };

        match accessor {
            Some(accessor) => accessor,
            None => {
                self.reports.push(
                    Report::error(Header::ExpectedExpression())
                        .with_primary_label(Label::Empty, self.loc_here()),
                );
                ast::Expr::Missing(ast::Lexeme {
                    span: self.span_here(),
                })
            }
        }
    }

    fn try_parse_int_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::Int)
            .map(|token| ast::Expr::Int(ast::Lexeme { span: token }))
    }

    fn try_parse_float_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::Float)
            .map(|token| ast::Expr::Float(ast::Lexeme { span: token }))
    }

    fn try_parse_string_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::String)
            .map(|token| ast::Expr::String(ast::Lexeme { span: token }))
    }

    fn try_parse_true_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::True)
            .map(|token| ast::Expr::True(ast::Lexeme { span: token }))
    }

    fn try_parse_false_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::False)
            .map(|token| ast::Expr::False(ast::Lexeme { span: token }))
    }

    fn try_parse_var_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::Ident)
            .map(|token| ast::Expr::Var(ast::Lexeme { span: token }))
    }

    fn try_parse_spread_expression(&mut self) -> Option<ast::Expr> {
        let spread = self.try_expect_token(Token::Spread)?;
        let name = self.try_expect_token(Token::Ident);

        Some(ast::Expr::Spread(ast::Spread { spread, name }))
    }

    fn try_parse_tuple_expression(&mut self) -> Option<ast::Expr> {
        let left_paren = self.try_expect_token(Token::LeftParen)?;
        let items = self.parse_comma_separated_items();
        let right_paren = self.expect_token(Token::RightParen);

        Some(ast::Expr::Tuple(ast::Tuple {
            left_paren,
            right_paren,
            items,
        }))
    }

    fn try_parse_array_expression(&mut self) -> Option<ast::Expr> {
        let left_bracket = self.try_expect_token(Token::LeftBracket)?;
        let items = self.parse_comma_separated_items();
        let right_bracket = self.expect_token(Token::RightBracket);

        Some(ast::Expr::Array(ast::Array {
            left_bracket,
            right_bracket,
            items,
        }))
    }

    fn try_parse_block_expression(&mut self) -> Option<ast::Expr> {
        let do_kw = self.try_expect_token(Token::Do)?;
        let label = self.parse_optional_label();
        let items = self.parse_newline_separated_items();
        let end_kw = self.expect_token(Token::End);

        Some(ast::Expr::Block(ast::Block {
            do_kw,
            end_kw,
            label: Box::new(label),
            items,
        }))
    }

    fn try_parse_conditional_expression(&mut self) -> Option<ast::Expr> {
        let first_branch = self.try_parse_non_empty_branch()?;

        let mut else_branches = Vec::new();
        while let Some(else_kw) = self.try_expect_token(Token::Else) {
            let else_branch = self.try_parse_branch();
            else_branches.push((else_kw, else_branch));
        }

        let end_kw = self.expect_token(Token::End);

        Some(ast::Expr::Conditional(ast::Conditional {
            first_branch: Box::new(first_branch),
            else_branches: else_branches.into(),
            end_kw,
        }))
    }

    fn try_parse_non_empty_branch(&mut self) -> Option<ast::Branch> {
        match self.peek() {
            Token::If => self.try_parse_if_branch(),
            Token::While => self.try_parse_while_branch(),
            Token::Loop => self.try_parse_loop_branch(),
            Token::Match => self.try_parse_match_branch(),
            _ => None,
        }
    }

    fn try_parse_branch(&mut self) -> ast::Branch {
        if let Some(branch) = self.try_parse_non_empty_branch() {
            return branch;
        };

        let label = self.parse_optional_label();
        let body = self.parse_newline_separated_items();

        ast::Branch::Else(ast::ElseBranch {
            label: Box::new(label),
            body,
        })
    }

    fn try_parse_if_branch(&mut self) -> Option<ast::Branch> {
        let if_kw = self.try_expect_token(Token::If)?;
        let label = self.parse_optional_label();
        let condition = self.expect_expression();
        let then_kw = self.expect_token(Token::Then);
        let body = self.parse_newline_separated_items();

        Some(ast::Branch::If(ast::IfBranch {
            if_kw,
            then_kw,
            label: Box::new(label),
            condition: Box::new(condition),
            body,
        }))
    }

    fn try_parse_while_branch(&mut self) -> Option<ast::Branch> {
        let while_kw = self.try_expect_token(Token::While)?;
        let label = self.parse_optional_label();
        let condition = self.expect_expression();
        let do_kw = self.expect_token(Token::Do);
        let body = self.parse_newline_separated_items();

        Some(ast::Branch::While(ast::WhileBranch {
            while_kw,
            do_kw,
            label: Box::new(label),
            condition: Box::new(condition),
            body,
        }))
    }

    fn try_parse_loop_branch(&mut self) -> Option<ast::Branch> {
        let loop_kw = self.try_expect_token(Token::Loop)?;
        let label = self.parse_optional_label();
        let items = self.parse_newline_separated_items();

        Some(ast::Branch::Loop(ast::LoopBranch {
            loop_kw,
            label: Box::new(label),
            body: items,
        }))
    }

    fn try_parse_match_branch(&mut self) -> Option<ast::Branch> {
        let match_kw = self.try_expect_token(Token::Match)?;
        let label = self.parse_optional_label();
        let scrutinee = self.expect_expression();
        let with_kw = self.expect_token(Token::With);

        let mut cases = Vec::new();
        self.skip_newlines();
        loop {
            let Some(pattern) = self.try_parse_primary_expression() else {
                break;
            };
            let maps = self.expect_token(Token::Maps);
            let value = self.expect_expression();

            cases.push(ast::MatchCase {
                maps,
                pattern: Box::new(pattern),
                value: Box::new(value),
            });

            if !self.skip_newlines() {
                break;
            }
        }

        Some(ast::Branch::Match(ast::MatchBranch {
            match_kw,
            with_kw,
            label: Box::new(label),
            scrutinee: Box::new(scrutinee),
            cases: cases.into(),
        }))
    }

    fn try_parse_break_expression(&mut self) -> Option<ast::Expr> {
        let break_kw = self.try_expect_token(Token::Break)?;
        let label = self.parse_optional_label();
        let expr = self.try_parse_expression().map(Box::new);

        Some(ast::Expr::Break(ast::Break {
            break_kw,
            label: Box::new(label),
            expr,
        }))
    }

    fn try_parse_skip_expression(&mut self) -> Option<ast::Expr> {
        let skip_kw = self.try_expect_token(Token::Skip)?;
        let label = self.parse_optional_label();

        Some(ast::Expr::Skip(ast::Skip {
            skip_kw,
            label: Box::new(label),
        }))
    }

    fn try_parse_let_expression(&mut self) -> Option<ast::Expr> {
        let let_kw = self.try_expect_token(Token::Let)?;
        let pattern = self.expect_primary_expression();
        let (assign, value) = self.parse_optional_symbol_then_expression(Token::Assign);

        Some(ast::Expr::Let(ast::Let {
            let_kw,
            assign,
            pattern: Box::new(pattern),
            value: Box::new(value),
        }))
    }

    fn try_parse_fun_expression(&mut self) -> Option<ast::Expr> {
        let fun_kw = self.try_expect_token(Token::Fun)?;
        let signature = self.expect_primary_expression();
        let (maps, value) = self.parse_optional_symbol_then_expression(Token::Maps);

        Some(ast::Expr::Fun(ast::Fun {
            fun_kw,
            maps,
            signature: Box::new(signature),
            value: Box::new(value),
        }))
    }

    fn parse_optional_symbol_then_expression(&mut self, token: Token) -> (Option<Span>, ast::Expr) {
        if let Some(e) = self.try_parse_block_expression() {
            (None, e)
        } else if let Some(e) = self.try_parse_conditional_expression() {
            (None, e)
        } else {
            (Some(self.expect_token(token)), self.expect_expression())
        }
    }

    fn try_parse_import_expression(&mut self) -> Option<ast::Expr> {
        let import_kw = self.try_expect_token(Token::Import)?;
        let queries = self.parse_strictly_comma_separated_items();

        if queries.is_empty() {
            self.reports.push(
                Report::error(Header::EmptyImport())
                    .with_primary_label(Label::Empty, import_kw.wrap(self.file))
                    .with_secondary_label(Label::ExpectedImportQuery, self.loc_here()),
            );
        }

        Some(ast::Expr::Import(ast::Import { import_kw, queries }))
    }

    fn try_parse_super_expression(&mut self) -> Option<ast::Expr> {
        self.try_expect_token(Token::Super)
            .map(|token| ast::Expr::Super(ast::Lexeme { span: token }))
    }

    fn parse_optional_label(&mut self) -> ast::Label {
        let Some(left_chev) = self.try_expect_token(Token::LeftChev) else {
            return ast::Label::Empty(self.span_here());
        };

        let name_expr = self.expect_expression();
        let right_chev = self.expect_token(Token::RightChev);

        ast::Label::Named(ast::NamedLabel {
            left_chev,
            right_chev,
            name_expr: Box::new(name_expr),
        })
    }

    fn parse_comma_separated_items(&mut self) -> Box<[ast::Expr]> {
        let mut items = Vec::new();

        self.skip_newlines();
        while let Some(item) = self.try_parse_expression() {
            items.push(item);

            if self.try_expect_token(Token::Comma).is_some() {
                self.skip_newlines();
            } else if !self.skip_newlines() {
                break;
            }
        }

        items.into()
    }

    fn parse_strictly_comma_separated_items(&mut self) -> Box<[ast::Expr]> {
        let mut items = Vec::new();

        while let Some(item) = self.try_parse_expression() {
            items.push(item);
            if self.try_expect_token(Token::Comma).is_none() {
                break;
            }
        }

        items.into()
    }

    fn parse_newline_separated_items(&mut self) -> Box<[ast::Expr]> {
        let mut items = Vec::new();

        self.skip_newlines();
        while let Some(item) = self.try_parse_expression() {
            items.push(item);
            if !self.skip_newlines() {
                break;
            }
        }

        items.into()
    }
}

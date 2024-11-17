use codespan_reporting::diagnostic::Severity;
use logos::{Logos, Span, SpannedIter};
use std::iter::Peekable;

use super::{
    ast::{self},
    reporting::{Header, Label, Report},
    Token,
};

pub struct Parser<'src, 'e> {
    source: &'src str,
    lexer: Peekable<SpannedIter<'src, Token>>,
    bounds: (usize, usize),
    reports: &'e mut Vec<Report<'src>>,
}

impl<'src, 'e> Parser<'src, 'e> {
    pub fn new(source: &'src str, reports: &'e mut Vec<Report<'src>>) -> Parser<'src, 'e> {
        let mut p = Parser {
            source,
            lexer: Token::lexer(source).spanned().peekable(),
            bounds: (0, 0),
            reports,
        };

        p.peek();
        p
    }

    fn pos_from(&self) -> usize {
        self.bounds.1
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
            Report::error(Header::InvalidCharacterSequence(&self.source[span.clone()]))
                .with_label(Label::Empty, span),
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
        let ((_, tok_span), start) = match self.lexer.next() {
            Some((tok, span)) => ((tok.unwrap(), span.clone()), span.end),
            None => ((Token::Eof, len..len), len),
        };

        self.bounds.0 = start;
        self.peek();
        tok_span
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
            let span = self.pos_from()..self.pos_from();
            self.reports.push(
                Report::error(Header::ExpectedToken(expected, p))
                    .with_label(Label::Empty, span.clone()),
            );
            span
        }
    }

    pub fn parse_program(&mut self) -> ast::Expr {
        self.try_expect_token(Token::Newline);
        let expr = self.expect_expression();
        self.try_expect_token(Token::Newline);
        self.expect_token(Token::Eof);

        self.reports.push(
            Report::new(
                Severity::Note,
                Header::Internal("parsed expression".to_string()),
            )
            .with_label(Label::Empty, expr.span()),
        );
        expr
    }

    pub fn expect_expression(&mut self) -> ast::Expr {
        match self.try_parse_primary_expression() {
            Some(expr) => expr,
            None => {
                let span = self.pos_from()..self.pos_from();
                self.reports.push(
                    Report::error(Header::ExpectedExpression())
                        .with_label(Label::Empty, span.clone()),
                );
                ast::Expr::Missing(ast::Lexeme { span })
            }
        }
    }

    pub fn try_parse_expression(&mut self) -> Option<ast::Expr> {
        self.try_parse_primary_expression()
    }

    pub fn try_parse_primary_expression(&mut self) -> Option<ast::Expr> {
        let expr = match self.peek() {
            Token::Int => self.try_parse_int_expression(),
            Token::Float => self.try_parse_float_expression(),
            Token::String => self.try_parse_string_expression(),
            Token::True => self.try_parse_true_expression(),
            Token::False => self.try_parse_false_expression(),
            Token::LeftParen => self.try_parse_tuple_expression(),
            Token::LeftBracket => self.try_parse_array_expression(),
            _ => None,
        }?;

        Some(expr)
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

    fn parse_comma_separated_items(&mut self) -> Box<[ast::Expr]> {
        let mut items = Vec::new();

        self.try_expect_token(Token::Newline);
        while let Some(item) = self.try_parse_expression() {
            items.push(item);

            if self.try_expect_token(Token::Comma).is_some() {
                self.try_expect_token(Token::Newline);
            } else if self.try_expect_token(Token::Newline).is_none() {
                break;
            }
        }

        items.into()
    }
}

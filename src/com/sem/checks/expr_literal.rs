use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src> Checker<'src, '_> {
    pub fn read_source_int(&mut self, span: Span) -> Option<i64> {
        match span.lexeme(self.source).parse() {
            Ok(n) => Some(n),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidInteger())
                        .with_primary_label(Label::Empty, span.wrap(self.file)),
                );
                None
            }
        }
    }

    pub fn read_source_float(&mut self, span: Span) -> Option<f64> {
        match span.lexeme(self.source).parse() {
            Ok(n) => Some(n),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidFloat())
                        .with_primary_label(Label::Empty, span.wrap(self.file)),
                );
                None
            }
        }
    }

    pub fn read_source_string(&self, span: Span) -> &'src str {
        &self.source[(span.start + 1)..(span.end - 1)]
    }

    pub fn check_int(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        (
            self.read_source_int(e.span)
                .map(ir::Expr::Int)
                .unwrap_or(ir::Expr::Missing),
            self.create_type(ir::Type::Int, Some(e.span)),
        )
    }

    pub fn check_float(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        (
            self.read_source_float(e.span)
                .map(ir::Expr::Float)
                .unwrap_or(ir::Expr::Missing),
            self.create_type(ir::Type::Float, Some(e.span)),
        )
    }

    pub fn check_string(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        (
            ir::Expr::String(self.read_source_string(e.span).to_string()),
            self.create_type(ir::Type::String, Some(e.span)),
        )
    }

    pub fn check_bool(&mut self, e: &ast::Lexeme, b: bool) -> ir::CheckedExpr {
        (
            ir::Expr::Bool(b),
            self.create_type(ir::Type::Bool, Some(e.span)),
        )
    }
}

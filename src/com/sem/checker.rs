use crate::com::{
    ast::{self},
    reporting::{Header, Label, Report},
};

pub struct Checker<'src, 'e> {
    source: &'src str,
    file: usize,
    reports: &'e mut Vec<Report>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(source: &'src str, file: usize, reports: &'e mut Vec<Report>) -> Self {
        Self {
            source,
            file,
            reports,
        }
    }

    pub fn check_file(&mut self, ast: &ast::File) {
        _ = self.source;
        _ = self.file;

        for expr in &ast.0 {
            self.check_expression(expr);
        }
    }

    fn check_expression(&mut self, e: &ast::Expr) {
        use ast::Expr as E;
        match e {
            E::Missing(e) => self.check_missing(e),
            E::Int(e) => self.check_int(e),
            E::Float(e) => self.check_float(e),
            E::String(e) => self.check_string(e),
            E::True(e) => self.check_bool(e, true),
            E::False(e) => self.check_bool(e, false),
            E::Var(..) => todo!(),
            E::Tuple(e) => self.check_tuple(e),
            E::Array(e) => self.check_array(e),
            E::Block(e) => self.check_block(e),
            E::Loop(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Call(..) => todo!(),
            E::Access(..) => todo!(),
            E::Fun(..) => todo!(),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
            }
        }
    }

    fn check_missing(&mut self, _: &ast::Lexeme) {}

    fn check_int(&mut self, e: &ast::Lexeme) {
        let lexeme = e.span.lexeme(self.source);
        let Ok(_) = lexeme.parse::<i64>() else {
            self.reports.push(
                Report::error(Header::InvalidInteger())
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return;
        };
    }

    fn check_float(&mut self, e: &ast::Lexeme) {
        let lexeme = e.span.lexeme(self.source);
        let Ok(_) = lexeme.parse::<f64>() else {
            self.reports.push(
                Report::error(Header::InvalidFloat())
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return;
        };
    }

    fn check_string(&mut self, _: &ast::Lexeme) {}

    fn check_bool(&mut self, _: &ast::Lexeme, _: bool) {}

    fn check_tuple(&mut self, e: &ast::Tuple) {
        for item in &e.items {
            self.check_expression(item);
        }
    }

    fn check_array(&mut self, e: &ast::Array) {
        for item in &e.items {
            self.check_expression(item);
        }
    }

    fn check_block(&mut self, e: &ast::Block) {
        for item in &e.items {
            self.check_expression(item);
        }
    }
}

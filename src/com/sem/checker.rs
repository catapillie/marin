use crate::com::{ast, reporting::Report};

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
        _ = self.reports;

        for expr in &ast.0 {
            self.check_expression(expr);
        }
    }

    pub fn check_expression(&mut self, expr: &ast::Expr) {
        // use ast::Expr as E;
        // match expr {
        //     E::Missing(..) => todo!(),
        //     E::Int(..) => todo!(),
        //     E::Float(..) => todo!(),
        //     E::String(..) => todo!(),
        //     E::True(..) => todo!(),
        //     E::False(..) => todo!(),
        //     E::Var(..) => todo!(),
        //     E::Tuple(..) => todo!(),
        //     E::Array(..) => todo!(),
        //     E::Spread(..) => todo!(),
        //     E::Block(..) => todo!(),
        //     E::Loop(..) => todo!(),
        //     E::Conditional(..) => todo!(),
        //     E::Break(..) => todo!(),
        //     E::Skip(..) => todo!(),
        //     E::Call(..) => todo!(),
        //     E::Access(..) => todo!(),
        //     E::Let(..) => todo!(),
        //     E::Fun(..) => todo!(),
        //     E::Import(..) => todo!(),
        // }
        _ = expr;
    }
}

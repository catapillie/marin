use crate::com::{ast, reporting::Report};

pub struct Checker<'src, 'e> {
    source: &'src str,
    reports: &'e mut Vec<Report>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(source: &'src str, reports: &'e mut Vec<Report>) -> Self {
        Self { source, reports }
    }

    pub fn check_module(&mut self, expr: &ast::Expr) {
        _ = self.reports;
        _ = self.source;
        use ast::Expr as E;
        match expr {
            E::Missing(..) => todo!(),
            E::Int(..) => todo!(),
            E::Float(..) => todo!(),
            E::String(..) => todo!(),
            E::True(..) => todo!(),
            E::False(..) => todo!(),
            E::Var(..) => todo!(),
            E::Tuple(..) => todo!(),
            E::Array(..) => todo!(),
            E::Spread(..) => todo!(),
            E::Block(..) => todo!(),
            E::Loop(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Call(..) => todo!(),
            E::Access(..) => todo!(),
            E::Let(..) => todo!(),
            E::Fun(..) => todo!(),
            E::Import(..) => todo!(),
        }
    }
}

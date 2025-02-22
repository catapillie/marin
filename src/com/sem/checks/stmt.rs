use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_statement(&mut self, e: &ast::Expr) -> ir::Stmt {
        use ast::Expr as E;
        match e {
            E::Pub(e) => match &*e.expr {
                E::Let(e) => self.check_let(e),
                E::Alias(e) => self.check_alias(e),
                E::Record(e) => self.check_record(e),
                E::Union(e) => self.check_union(e),
                E::Class(e) => self.check_class(e),
                E::Have(e) => self.check_have(e),
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidExpression())
                            .with_primary_label(Label::Empty, e.expr.span().wrap(self.file))
                            .with_secondary_label(Label::PublicStatement, e.span().wrap(self.file))
                            .with_note(Note::PubExpression),
                    );
                    ir::Stmt::Missing
                }
            },

            E::Let(e) => self.check_let(e),
            E::Alias(e) => self.check_alias(e),
            E::Import(e) => self.check_import(e),
            E::Record(e) => self.check_record(e),
            E::Union(e) => self.check_union(e),
            E::Class(e) => self.check_class(e),
            E::Have(e) => self.check_have(e),
            _ => {
                let (expr, ty) = self.check_expression(e);
                ir::Stmt::Expr(expr, ty)
            }
        }
    }
}

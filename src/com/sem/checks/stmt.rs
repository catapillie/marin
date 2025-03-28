use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_statement(&mut self, e: &ast::Expr) -> ir::Stmt {
        use ast::Expr as E;
        match e {
            E::Pub(e) => match &*e.expr {
                E::Let(e) => self.check_let(e, true),
                E::Alias(e) => self.check_alias(e, true),
                E::Import(e) => self.check_import(e, true),
                E::ImportFrom(e) => self.check_import_from(e, true),
                E::Record(e) => self.check_record(e, true),
                E::Union(e) => self.check_union(e, true),
                E::Class(e) => self.check_class(e, true),
                E::Have(e) => self.check_have(e, true),
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

            E::Let(e) => self.check_let(e, false),
            E::Alias(e) => self.check_alias(e, false),
            E::Import(e) => self.check_import(e, false),
            E::ImportFrom(e) => self.check_import_from(e, false),
            E::Record(e) => self.check_record(e, false),
            E::Union(e) => self.check_union(e, false),
            E::Class(e) => self.check_class(e, false),
            E::Have(e) => self.check_have(e, false),
            _ => {
                let (expr, ty) = self.check_expression(e);
                ir::Stmt::Expr(expr, ty)
            }
        }
    }
}

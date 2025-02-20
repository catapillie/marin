use crate::com::{
    ast,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

use super::path::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_identifier_accessor(&mut self, e: &ast::Expr) -> Option<(&'src str, Span)> {
        use ast::Expr as E;
        match e {
            E::Var(e) => Some((e.span.lexeme(self.source), e.span)),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidAccessor())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                None
            }
        }
    }

    pub fn check_access_path(&mut self, e: &ast::Access) -> Q {
        let q = self.check_path(&e.accessed);
        match q {
            Q::Missing => Q::Missing,
            Q::Expr(_) => todo!("access on expr"),
            Q::Type(_) => todo!("access on type"),
            Q::Record(id) => self.check_record_access_path(id, &e.accessor),
            Q::Union(id) => self.check_union_access_path(id, &e.accessor),
            Q::Variant(_, _) => todo!("access on variant"),
            Q::Class(id) => self.check_class_access_path(id, &e.accessor, e.span()),
            Q::Import(id) => self.check_import_access_path(id, &e.accessor),
        }
    }
}

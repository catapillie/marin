use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

use super::path::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    fn check_identifier_accessor(&mut self, e: &ast::Expr) -> Option<(&'src str, Span)> {
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

    pub fn check_union_access_path(&mut self, id: ir::EntityID, accessor: &ast::Expr) -> Q {
        let Some((name, name_span)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let info = self.get_union_info(id);

        let Some((tag, _)) = info
            .variants
            .iter()
            .enumerate()
            .find(|(_, var)| var.name == name)
        else {
            self.reports.push(
                Report::error(Header::UnknownVariant(name.to_string(), info.name.clone()))
                    .with_primary_label(Label::Empty, name_span.wrap(self.file))
                    .with_secondary_label(Label::UnionDefinition(info.name.clone()), info.loc),
            );
            return Q::Missing;
        };

        Q::Variant(id, tag)
    }
}

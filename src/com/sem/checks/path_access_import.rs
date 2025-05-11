use crate::com::{
    Checker, ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
};

use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_import_access_path(
        &mut self,
        id: ir::ImportID,
        accessor: &ast::Expr,
        span: Span,
    ) -> Q {
        let Some((name, _)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let info = self.entities.get_import_info(id);
        let file = info.file;
        debug_assert_ne!(file, self.file, "should be unable to import from self");

        let items = &self.exports[file];
        if !items.was_checked {
            return Q::Missing;
        }

        match items.exports.get(name) {
            Some(id) => self.check_entity_into_path(*id),
            None => {
                self.reports.push(
                    Report::error(Header::UnknownExport(name.to_string(), info.name.clone()))
                        .with_primary_label(Label::Empty, span.wrap(self.file))
                        .with_secondary_label(Label::ImportedHere(info.name.clone()), info.loc),
                );
                Q::Missing
            }
        }
    }
}

use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_class_access_path(&mut self, id: ir::ClassID, accessor: &ast::Expr) -> Q {
        let Some((name, name_span)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let info = self.entities.get_class_info(id);

        let Some((item_id, _)) = info
            .items
            .iter()
            .enumerate()
            .find(|(_, item)| item.name == name)
        else {
            self.reports.push(
                Report::error(Header::UnknownClassItem(
                    name.to_string(),
                    info.name.clone(),
                ))
                .with_primary_label(Label::Empty, name_span.wrap(self.file))
                .with_secondary_label(Label::ClassDefinition(info.name.clone()), info.loc),
            );
            return Q::Missing;
        };

        Q::ClassItem(id, item_id)
    }
}

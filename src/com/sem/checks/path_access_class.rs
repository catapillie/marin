use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_class_access_path(
        &mut self,
        id: ir::EntityID,
        accessor: &ast::Expr,
        span: Span,
    ) -> Q {
        let Some((name, name_span)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let info = self.get_class_info(id);
        let class_loc = info.loc;
        let class_name = info.name.clone();

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

        let item_info = self.get_class_item_info(id, item_id);
        let item_loc = item_info.loc;
        let item_name = item_info.name.clone();

        let scheme = item_info.scheme.clone();

        let item_ty = self.instantiate_scheme(scheme, Some(span.wrap(self.file)));
        let item_ty = self.clone_type_repr(item_ty);
        self.set_type_span(item_ty, span);
        self.add_type_provenance(
            item_ty,
            ir::TypeProvenance::ClassItemDefinition(item_loc, item_name, class_loc, class_name),
        );

        Q::Expr((ir::Expr::Missing, item_ty))
    }
}

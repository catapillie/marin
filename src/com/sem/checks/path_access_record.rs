use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Report},
};

use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_record_access_path(&mut self, id: ir::RecordID, accessor: &ast::Expr) -> Q {
        let Some((name, name_span)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let info = self.entities.get_record_info(id);

        let Some((tag, _)) = info
            .fields
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

        let (info, field_info) = self.entities.get_record_field_info(id, tag);
        let getter_full_name = format!("{}.{}", info.name, field_info.name);

        let domain = info.scheme.forall.clone();
        let uninstantiated_record = info.scheme.uninstantiated;
        let uninstantiated_field = field_info.ty;
        let sub = self.build_type_substitution(domain);

        let record_type = self.apply_type_substitution(uninstantiated_record, &sub);
        let field_type = self.apply_type_substitution(uninstantiated_field, &sub);

        let arg_id = self.entities.create_dummy_variable();
        let getter_expr = ir::Expr::Fun {
            name: getter_full_name,
            recursive_binding: None,
            signature: Box::new(ir::Signature::Args {
                args: Box::new([ir::Pattern::Binding(arg_id)]),
                next: Box::new(ir::Signature::Done),
            }),
            expr: Box::new(ir::Expr::Access {
                accessed: Box::new(ir::Expr::Var { id: arg_id }),
                index: tag,
            }),
        };

        let getter_type =
            self.create_type(ir::Type::Lambda(Box::new([record_type]), field_type), None);

        Q::Expr((getter_expr, getter_type))
    }
}

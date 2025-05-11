use crate::com::{Checker, ast, ir, sem::provenance::Provenance};

impl Checker<'_, '_> {
    pub fn check_array(&mut self, e: &ast::Array) -> ir::CheckedExpr {
        let array_item_type = self.create_fresh_type(None);
        let (items, item_types) = self.check_expression_list(&e.items);

        let provenances = &[Provenance::ArrayItems(e.span().wrap(self.file))];
        for item_type in item_types {
            self.unify(item_type, array_item_type, provenances);
        }

        (
            ir::Expr::Array {
                items: items.into(),
            },
            self.create_type(ir::Type::Array(array_item_type), Some(e.span())),
        )
    }
}

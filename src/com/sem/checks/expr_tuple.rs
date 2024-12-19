use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_tuple(&mut self, e: &ast::Tuple) -> ir::CheckedExpr {
        if e.items.len() == 1 {
            return self.check_expression(&e.items[0]);
        }

        let (items, item_types) = self.check_expression_list(&e.items);
        (
            ir::Expr::Tuple(items.into()),
            self.create_type(ir::Type::Tuple(item_types.into()), Some(e.span())),
        )
    }
}

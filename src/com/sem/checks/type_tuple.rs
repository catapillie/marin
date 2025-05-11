use crate::com::{Checker, ast, ir};

impl Checker<'_, '_> {
    pub fn check_tuple_type(&mut self, t: &ast::Tuple) -> ir::TypeID {
        if t.items.len() == 1 {
            return self.check_type(&t.items[0]);
        }

        let item_types = t.items.iter().map(|item| self.check_type(item)).collect();
        self.create_type(ir::Type::Tuple(item_types), Some(t.span()))
    }
}

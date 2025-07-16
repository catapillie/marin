use crate::com::{Checker, ast, ir};

impl Checker<'_, '_> {
    pub fn check_array_type(&mut self, t: &ast::ArrayType) -> ir::TypeID {
        let item_type = self.check_type(&t.ty);
        self.create_type(ir::Type::Array(item_type), Some(t.span()))
    }
}

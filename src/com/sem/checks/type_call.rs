use crate::com::{Checker, ast, ir};

impl Checker<'_, '_> {
    pub fn check_call_type(&mut self, t: &ast::Call) -> ir::TypeID {
        let q = self.check_call_path(t);
        self.check_path_into_type(q, t.span())
    }
}

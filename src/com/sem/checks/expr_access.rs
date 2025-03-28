use crate::com::{ast, ir, Checker};

impl Checker<'_, '_> {
    pub fn check_access(&mut self, e: &ast::Access) -> ir::CheckedExpr {
        let q = self.check_access_path(e);
        self.check_path_into_expr(q, e.span())
    }
}

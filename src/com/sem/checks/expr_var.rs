use crate::com::{
    Checker, ast,
    ir::{self},
};

impl Checker<'_, '_> {
    pub fn check_var(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let q = self.check_var_path(e);
        self.check_path_into_expr(q, e.span)
    }
}

use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_var_type(&mut self, t: &ast::Lexeme) -> ir::TypeID {
        let q = self.check_var_path(t);
        self.check_path_into_type(q, t.span)
    }
}

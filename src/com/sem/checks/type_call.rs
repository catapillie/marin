use crate::com::{
    ast, ir,
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_call_type(&mut self, t: &ast::Call) -> ir::TypeID {
        let q = self.check_call_path(t);
        self.check_path_into_type(q, t.span())
    }
}

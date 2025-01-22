use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_fun_type(&mut self, t: &ast::Fun) -> ir::TypeID {
        let type_sig = self.check_type_signature(&t.signature);
        let (lam_type, sig_ret_type) = self.declare_type_signature(&type_sig);

        let ret_type = self.check_type(&t.value);
        self.unify(ret_type, sig_ret_type, &[]);

        lam_type
    }
}

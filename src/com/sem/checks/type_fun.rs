use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_fun_type(&mut self, t: &ast::Fun) -> ir::TypeID {
        let (type_sig, sig_name_span) = self.check_type_signature(&t.signature);
        let (lam_type, sig_ret_type) = self.declare_type_signature(&type_sig);

        if let Some(span) = sig_name_span {
            self.reports.push(
                Report::warning(Header::UnallowedSignatureName()).with_primary_label(
                    Label::FunctionTypeCannotHaveName(span.lexeme(self.source).to_string()),
                    t.span().wrap(self.file),
                ),
            );
        }

        let ret_type = self.check_type(&t.value);
        self.unify(ret_type, sig_ret_type, &[]);

        lam_type
    }
}

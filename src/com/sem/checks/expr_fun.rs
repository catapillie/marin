use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_fun(&mut self, e: &ast::Fun) -> ir::CheckedExpr {
        let signature = self.check_signature(&e.signature, true);
        let sig_span = Span::combine(e.fun_kw, e.signature.span());
        for arg_pattern in signature.arg_patterns() {
            if !arg_pattern.is_irrefutable() {
                self.reports.push(
                    Report::error(Header::RefutablePattern())
                        .with_primary_label(Label::Empty, arg_pattern.span().wrap(self.file))
                        .with_secondary_label(Label::FunctionArgPattern, sig_span.wrap(self.file)),
                );
            }
        }

        let prev_fun_info = self.create_function_info();
        self.open_scope(true);

        let fun_uid_name = format!("fun_{}", self.get_generic_unique_id());
        self.set_scope_name(fun_uid_name);

        let (sig, sig_type, ret_type, id) = self.declare_signature(&signature);
        self.set_type_span(sig_type, sig_span);
        self.set_type_span(ret_type, e.value.span());

        let (val, val_type) = self.check_expression(&e.value);
        self.unify(val_type, ret_type, &[]);

        let fun_name = self.build_scope_name();
        self.close_scope();
        let fun_info = self.restore_function_info(prev_fun_info);

        (
            ir::Expr::Fun(fun_name, id, fun_info, Box::new(sig), Box::new(val)),
            sig_type,
        )
    }
}

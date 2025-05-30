use crate::com::{Checker, ast, ir, sem::provenance::Provenance};

impl Checker<'_, '_> {
    pub fn check_call(&mut self, e: &ast::Call) -> ir::CheckedExpr {
        let (callee, callee_type) = self.check_expression(&e.callee);
        let (args, arg_types) = self.check_expression_list(&e.args);

        let ret_type = self.create_fresh_type(Some(e.span()));
        let sig_type = self.create_type(ir::Type::Lambda(arg_types.into(), ret_type), None);

        let provenances = &[Provenance::FunctionCall(
            self.get_type_string(sig_type),
            e.callee.span().wrap(self.file),
        )];
        self.unify(callee_type, sig_type, provenances);

        let result_ty = self.clone_type_repr(ret_type);
        self.set_type_span(result_ty, e.span());

        (
            ir::Expr::Call {
                callee: Box::new(callee),
                args: args.into(),
            },
            result_ty,
        )
    }
}

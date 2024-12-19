use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_call(&mut self, e: &ast::Call) -> ir::CheckedExpr {
        let (callee, callee_type) = self.check_expression(&e.callee);
        let (args, arg_types) = self.check_expression_list(&e.args);

        let ret_type = self.create_fresh_type(Some(e.span()));
        let sig_type = self.create_type(
            ir::Type::Lambda(arg_types.into(), ret_type),
            Some(e.callee.span()),
        );
        self.unify(callee_type, sig_type, &[]);

        (ir::Expr::Call(Box::new(callee), args.into()), ret_type)
    }
}

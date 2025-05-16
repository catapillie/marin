use crate::com::{Checker, ast, ir};
use std::collections::HashMap;

impl Checker<'_, '_> {
    pub fn check_binary(&mut self, e: &ast::Binary) -> ir::CheckedExpr {
        use ast::BinOp as Op;
        if matches!(e.op, Op::And | Op::Or | Op::Xor) {
            return self.check_boolean_binary(e);
        }

        let (left, left_ty) = self.check_expression(&e.left);
        let (right, right_ty) = self.check_expression(&e.right);

        let Some(prelude_file) = self.deps.info.prelude_file else {
            panic!("cannot use binary operation without std")
        };

        let (op_class_name, op_class_item) = match e.op {
            Op::Add => ("Add", 0),
            Op::Sub => ("Sub", 0),
            Op::Mul => ("Mul", 0),
            Op::Div => ("Div", 0),
            Op::Mod => ("Mod", 0),
            Op::Eq => ("Eq", 0),  // Eq.eq
            Op::Ne => ("Eq", 1),  // Eq.ne
            Op::Lt => ("Ord", 0), // Ord.lt
            Op::Le => ("Ord", 1), // Ord.le
            Op::Gt => ("Ord", 2), // Ord.gt
            Op::Ge => ("Ord", 3), // Ord.ge
            Op::BitXor => ("BitXor", 0),
            Op::BitOr => ("BitOr", 0),
            Op::BitAnd => ("BitAnd", 0),
            op => unimplemented!("binary operator '{op:?}'"),
        };

        let ops_exports = self.get_marin_std_ops_exports(prelude_file);
        let Some(ir::AnyID::Class(op_class_id)) = ops_exports.get(op_class_name).copied() else {
            panic!(
                "couldn't find op.{op_class_name} class for binary operator {:?}",
                e.op
            );
        };

        let (op, op_ty) = self.check_class_item_into_expr(op_class_id, op_class_item, e.op_tok);

        let ret_ty = self.create_fresh_type(Some(e.span()));
        let expected_op_ty = self.create_type(
            ir::Type::Lambda(Box::new([left_ty, right_ty]), ret_ty),
            Some(e.op_tok),
        );

        self.unify(op_ty, expected_op_ty, &[]);

        let result_ty = self.clone_type_repr(ret_ty);
        self.set_type_span(result_ty, e.span());

        (
            ir::Expr::Call {
                callee: Box::new(op),
                args: Box::new([left, right]),
            },
            result_ty,
        )
    }

    pub fn check_unary(&mut self, e: &ast::Unary) -> ir::CheckedExpr {
        use ast::UnOp as Op;
        if matches!(e.op, Op::Not) {
            return self.check_boolean_unary(e);
        }

        let (arg, arg_ty) = self.check_expression(&e.arg);

        let Some(prelude_file) = self.deps.info.prelude_file else {
            panic!("cannot use binary operation without std")
        };
        let (op_class_name, op_class_item) = match e.op {
            Op::Pos => ("Pos", 0),
            Op::Neg => ("Neg", 0),
            Op::BitNeg => ("BitNeg", 0),
            op => unimplemented!("unary operator '{op:?}'"),
        };

        let ops_exports = self.get_marin_std_ops_exports(prelude_file);
        let Some(ir::AnyID::Class(op_class_id)) = ops_exports.get(op_class_name).copied() else {
            panic!(
                "couldn't find op.{op_class_name} class for unary operator {:?}",
                e.op
            );
        };

        let (op, op_ty) = self.check_class_item_into_expr(op_class_id, op_class_item, e.op_tok);

        let ret_ty = self.create_fresh_type(Some(e.span()));
        let expected_op_ty =
            self.create_type(ir::Type::Lambda(Box::new([arg_ty]), ret_ty), Some(e.op_tok));

        self.unify(op_ty, expected_op_ty, &[]);

        let result_ty = self.clone_type_repr(ret_ty);
        self.set_type_span(result_ty, e.span());

        (
            ir::Expr::Call {
                callee: Box::new(op),
                args: Box::new([arg]),
            },
            result_ty,
        )
    }

    fn get_marin_std_ops_exports(&mut self, prelude_file: usize) -> &HashMap<&str, ir::AnyID> {
        let prelude_exports = &self.exports[prelude_file].exports;
        let Some(ir::AnyID::Import(ops_import_id)) = prelude_exports.get("ops").copied() else {
            panic!("couldn't find 'ops' import in 'std.prelude'")
        };
        let ops_import_info = self.entities.get_import_info(ops_import_id);
        let ops_file = ops_import_info.file;
        &self.exports[ops_file].exports
    }

    fn check_boolean_binary(&mut self, e: &ast::Binary) -> ir::CheckedExpr {
        let bool_ty = self.native_types.bool;

        let (left, left_ty) = self.check_expression(&e.left);
        let (right, right_ty) = self.check_expression(&e.right);

        self.unify(left_ty, bool_ty, &[]);
        self.unify(right_ty, bool_ty, &[]);

        let expr = match e.op {
            ast::BinOp::And => ir::Expr::ShortAnd(Box::new(left), Box::new(right)),
            ast::BinOp::Or => ir::Expr::ShortOr(Box::new(left), Box::new(right)),
            ast::BinOp::Xor => ir::Expr::BitXor(Box::new(left), Box::new(right)),
            _ => unreachable!("unreachable boolean binary operation"),
        };

        (expr, bool_ty)
    }

    fn check_boolean_unary(&mut self, e: &ast::Unary) -> ir::CheckedExpr {
        let bool_ty = self.native_types.bool;

        let (arg, arg_ty) = self.check_expression(&e.arg);
        self.unify(arg_ty, bool_ty, &[]);

        let expr = match e.op {
            ast::UnOp::Not => ir::Expr::BitNeg(Box::new(arg)),
            _ => unreachable!("unreachable boolean unary operation"),
        };

        (expr, bool_ty)
    }
}

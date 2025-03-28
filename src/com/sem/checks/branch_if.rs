use crate::com::{ast, ir, loc::Span, sem::provenance::Provenance, Checker};

impl Checker<'_, '_> {
    pub fn check_if(&mut self, b: &ast::IfBranch, span: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (condition, condition_type) = self.check_expression(&b.condition);
        let bool_type = self.create_type(ir::Type::Bool, None);
        let provenances = &[Provenance::ConditionalBoolType(
            b.condition.span().wrap(self.file),
        )];
        self.unify(condition_type, bool_type, provenances);

        let (stmts, label_id, branch_type) = self.check_expression_block(&b.label, &b.body, span);
        let branch = ir::Branch::If(Box::new(condition), stmts, label_id);
        (branch, branch_type, false)
    }
}

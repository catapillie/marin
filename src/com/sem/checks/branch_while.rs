use crate::com::{ast, ir, loc::Span, sem::provenance::Provenance, Checker};

impl Checker<'_, '_> {
    pub fn check_while(&mut self, b: &ast::WhileBranch, _: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (condition, condition_type) = self.check_expression(&b.condition);
        let bool_type = self.create_type(ir::Type::Bool, None);
        let provenances = &[Provenance::ConditionalBoolType(
            b.condition.span().wrap(self.file),
        )];
        self.unify(condition_type, bool_type, provenances);

        let (stmts, label_id, branch_type) = self.check_statement_block(&b.label, &b.body, true);
        let branch = ir::Branch::While {
            guard: Box::new(condition),
            body: stmts,
            label: label_id,
        };
        (branch, branch_type, false)
    }
}

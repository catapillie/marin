use crate::com::{ast, ir, loc::Span, Checker};

impl Checker<'_, '_> {
    pub fn check_else(
        &mut self,
        b: &ast::ElseBranch,
        span: Span,
    ) -> (ir::Branch, ir::TypeID, bool) {
        let (stmts, label_id, branch_type) = self.check_expression_block(&b.label, &b.body, span);
        let branch = ir::Branch::Else {
            body: stmts,
            label: label_id,
        };
        (branch, branch_type, true)
    }
}

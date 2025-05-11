use crate::com::{Checker, ast, ir, loc::Span};

impl Checker<'_, '_> {
    pub fn check_loop(&mut self, e: &ast::LoopBranch, _: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (stmts, label_id, loop_type) = self.check_statement_block(&e.label, &e.body, true);
        let branch = ir::Branch::Loop {
            body: stmts,
            label: label_id,
        };
        (branch, loop_type, true)
    }
}

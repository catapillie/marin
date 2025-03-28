use crate::com::{ast, ir, Checker};

impl Checker<'_, '_> {
    // returns (branch_type, is_exhaustive)
    pub fn check_branch(&mut self, b: &ast::Branch) -> (ir::Branch, ir::TypeID, bool) {
        use ast::Branch as B;
        let span = b.span();
        match b {
            B::If(b) => self.check_if(b, span),
            B::While(b) => self.check_while(b, span),
            B::Loop(b) => self.check_loop(b, span),
            B::Else(b) => self.check_else(b, span),
            B::Match(b) => self.check_match(b, span),
        }
    }
}

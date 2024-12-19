use crate::com::{ast, ir, loc::Span, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_match(&mut self, b: &ast::MatchBranch, span: Span) -> (ir::Branch, ir::TypeID, bool) {
        let result_type = self.create_fresh_type(Some(span));

        let (scrut, scrut_type) = self.check_expression(&b.scrutinee);
        let mut is_exhaustive = false;
        let mut cases = Vec::new();
        for case in &b.cases {
            let pattern = self.check_pattern(&case.pattern);
            is_exhaustive |= pattern.is_irrefutable();

            self.open_scope(false);

            let (pattern, pattern_type) = self.declare_pattern(&pattern);
            self.unify(scrut_type, pattern_type, &[]);

            let (val, val_type) = self.check_expression(&case.value);
            self.unify(val_type, result_type, &[]);

            self.close_scope();
            cases.push((pattern, val));
        }

        (
            ir::Branch::Match(Box::new(scrut), cases.into()),
            result_type,
            is_exhaustive,
        )
    }
}

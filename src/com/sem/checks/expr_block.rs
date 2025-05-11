use crate::com::{Checker, ast, ir, loc::Span, sem::provenance::Provenance};

impl Checker<'_, '_> {
    pub fn check_expression_block(
        &mut self,
        label: &ast::Label,
        items: &[ast::Expr],
        span: Span,
    ) -> (Box<[ir::Stmt]>, ir::LabelID, ir::TypeID) {
        let mut iter = items.iter().peekable();
        let mut stmts = Vec::with_capacity(items.len());
        let mut last_type = self.create_type(ir::Type::unit(), Some(span));

        self.open_scope(false);
        let label_id = self.check_label_definition(label, false);

        while let Some(item) = iter.next() {
            let s = self.check_statement(item);
            if iter.peek().is_none() {
                if let ir::Stmt::Expr { expr: _, ty } = &s {
                    last_type = *ty;
                }
            }
            stmts.push(s);
        }

        self.close_scope();

        let label_info = self.get_label(label_id);
        let provenances = &[Provenance::LabelValues(
            label_info.loc,
            label_info.name.clone(),
        )];
        self.unify(last_type, label_info.ty, provenances);

        (stmts.into(), label_id, last_type)
    }

    pub fn check_statement_block(
        &mut self,
        label: &ast::Label,
        items: &[ast::Expr],
        skippable: bool,
    ) -> (Box<[ir::Stmt]>, ir::LabelID, ir::TypeID) {
        self.open_scope(false);
        let label_id = self.check_label_definition(label, skippable);

        let mut stmts = Vec::with_capacity(items.len());
        for item in items {
            let s = self.check_statement(item);
            stmts.push(s);
        }

        self.close_scope();

        (stmts.into(), label_id, self.get_label(label_id).ty)
    }

    pub fn check_block(&mut self, e: &ast::Block) -> ir::CheckedExpr {
        let (stmts, label_id, last_type) =
            self.check_expression_block(&e.label, &e.items, e.span());
        (
            ir::Expr::Block {
                stmts,
                label: label_id,
            },
            last_type,
        )
    }
}

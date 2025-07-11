use crate::com::{
    Checker, ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    sem::provenance::Provenance,
};

impl Checker<'_, '_> {
    pub fn check_index(&mut self, e: &ast::Index) -> ir::CheckedExpr {
        let (indexed, indexed_ty) = self.check_expression(&e.indexed);
        let mut checked_indices = e
            .indices
            .iter()
            .map(|item| self.check_expression(item))
            .collect::<Vec<_>>();

        let item_ty = self.create_fresh_type(None);
        let array_ty = self.create_type(ir::Type::Array(item_ty), None);
        let provenances = &[Provenance::IndexedMustBeArray(e.span().wrap(self.file))];
        self.unify(indexed_ty, array_ty, provenances);

        let indices_span = Span::combine(e.left_bracket, e.right_bracket);

        if checked_indices.len() > 1 {
            self.reports.push(
                Report::error(Header::InvalidIndexing())
                    .with_primary_label(
                        Label::UnsupportedMultidimensionalIndex,
                        indices_span.wrap(self.file),
                    )
                    .with_secondary_label(Label::Empty, e.span().wrap(self.file)),
            );
            return (ir::Expr::Missing, item_ty);
        }

        let Some((index, index_ty)) = checked_indices.pop() else {
            self.reports.push(
                Report::error(Header::InvalidIndexing())
                    .with_primary_label(Label::MissingIndex, indices_span.wrap(self.file))
                    .with_secondary_label(Label::Empty, e.span().wrap(self.file)),
            );
            return (ir::Expr::Missing, item_ty);
        };

        let provenances = &[Provenance::IndexMustBeInteger(indices_span.wrap(self.file))];
        self.unify(index_ty, self.native_types.int, provenances);

        (
            ir::Expr::Index {
                indexed: Box::new(indexed),
                index: Box::new(index),
            },
            item_ty,
        )
    }
}

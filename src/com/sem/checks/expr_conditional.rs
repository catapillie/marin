use crate::com::{
    ast,
    ir::{self, TypeProvenance},
    loc::Span,
    reporting::{Header, Label, Report},
    sem::provenance::Provenance,
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_conditional(&mut self, e: &ast::Conditional) -> ir::CheckedExpr {
        let mut branches = Vec::with_capacity(e.else_branches.len() + 1);
        let mut branch_types = Vec::with_capacity(e.else_branches.len() + 1);
        let (first_branch, first_branch_type, mut is_exhaustive) =
            self.check_branch(&e.first_branch);
        branches.push(first_branch);
        branch_types.push(first_branch_type);

        let mut exhaustive_branches_span = e.first_branch.span();
        let mut exhaustive_branch_count = 1;
        let mut unreachable_branches_span = None;
        let mut unreachable_branch_count = 0;
        for (else_tok, else_branch) in &e.else_branches {
            let branch_span = Span::combine(*else_tok, else_branch.span());
            let (branch, else_branch_type, is_else_exhaustive) = self.check_branch(else_branch);
            branch_types.push(else_branch_type);

            if !is_exhaustive {
                branches.push(branch);
                exhaustive_branches_span = Span::combine(exhaustive_branches_span, branch_span);
                exhaustive_branch_count += 1;
                is_exhaustive |= is_else_exhaustive;
            } else {
                unreachable_branch_count += 1;
                unreachable_branches_span = match unreachable_branches_span {
                    Some(sp) => Some(Span::combine(sp, branch_span)),
                    None => Some(branch_span),
                }
            }
        }

        if let Some(branches_span) = unreachable_branches_span {
            self.reports.push(
                Report::warning(Header::UnreachableConditionalBranches(
                    unreachable_branch_count,
                ))
                .with_primary_label(
                    Label::UnreachableConditionalBranches(unreachable_branch_count),
                    branches_span.wrap(self.file),
                )
                .with_secondary_label(
                    Label::ExhaustiveConditionalBranches(exhaustive_branch_count),
                    exhaustive_branches_span.wrap(self.file),
                ),
            );
        }

        let result_type = if is_exhaustive {
            let result_type = self.create_fresh_type(Some(e.span()));
            let provenances = &[Provenance::ConditionalReturnValues(
                e.span().wrap(self.file),
            )];
            for branch_type in branch_types {
                self.unify(branch_type, result_type, provenances);
            }
            result_type
        } else {
            let unit_type = self.create_type(ir::Type::unit(), Some(e.span()));
            self.add_type_provenance(
                unit_type,
                TypeProvenance::NonExhaustiveConditional(e.span().wrap(self.file)),
            );
            unit_type
        };

        (
            ir::Expr::Conditional(branches.into(), is_exhaustive),
            result_type,
        )
    }
}

use crate::com::{
    ast,
    ir::{self, TypeProvenance},
    reporting::{Header, Label, Report},
    sem::provenance::Provenance,
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_break(&mut self, e: &ast::Break) -> ir::CheckedExpr {
        let (value, ty) = e
            .expr
            .as_ref()
            .map(|val| {
                let (e, t) = self.check_expression(val);
                (Some(e), t)
            })
            .unwrap_or((None, self.create_type(ir::Type::unit(), Some(e.span()))));

        let label_name = self.check_label_name(&e.label);
        let Some(label_id) = self.find_label_by_name(label_name, false) else {
            let name = label_name.map(str::to_string);
            self.reports.push(
                Report::error(Header::InvalidBreak(name.clone()))
                    .with_primary_label(Label::NoBreakpointFound(name), e.span().wrap(self.file)),
            );
            return self.check_missing();
        };

        self.add_type_provenance(
            ty,
            TypeProvenance::ReturnedFromBreak(
                e.span().wrap(self.file),
                self.get_label(label_id).name.clone(),
            ),
        );

        let label_info = self.get_label(label_id);
        let provenances = &[Provenance::LabelValues(
            label_info.loc,
            label_info.name.clone(),
        )];
        self.unify(ty, label_info.ty, provenances);

        (
            ir::Expr::Break(value.map(Box::new), label_id),
            self.create_fresh_type(Some(e.span())),
        )
    }
}

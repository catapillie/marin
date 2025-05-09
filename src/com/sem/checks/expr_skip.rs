use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_skip(&mut self, e: &ast::Skip) -> ir::CheckedExpr {
        let label_name = self.check_label_name(&e.label);
        let Some(label_id) = self.find_label_by_name(label_name, true) else {
            let name = label_name.map(str::to_string);
            self.reports.push(
                Report::error(Header::InvalidSkip(name.clone()))
                    .with_primary_label(Label::NoSkippointFound(name), e.span().wrap(self.file)),
            );
            return self.check_missing();
        };

        if !self.get_label(label_id).skippable {
            let name = label_name.map(str::to_string);
            self.reports.push(
                Report::error(Header::InvalidSkip(name.clone()))
                    .with_primary_label(Label::Empty, e.span().wrap(self.file))
                    .with_secondary_label(
                        Label::UnskippableBlock(name),
                        self.get_label(label_id).loc,
                    ),
            );
        };

        (
            ir::Expr::Skip { label: label_id },
            self.create_fresh_type(Some(e.span())),
        )
    }
}

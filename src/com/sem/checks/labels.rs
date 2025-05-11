use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Report},
};

impl<'src> Checker<'src, '_> {
    fn add_label(&mut self, label: ir::Label) -> ir::LabelID {
        let id = self.labels.len();
        self.labels.push(label);
        ir::LabelID(id)
    }

    pub fn get_label(&self, id: ir::LabelID) -> &ir::Label {
        &self.labels[id.0]
    }

    pub fn check_label_definition(&mut self, l: &ast::Label, skippable: bool) -> ir::LabelID {
        let name = self.check_label_name(l);
        let ty = self.create_fresh_type(None);
        let id = self.add_label(ir::Label {
            name: name.map(str::to_string),
            ty,
            skippable,
            loc: l.span().wrap(self.file),
        });
        self.label_scope.insert(name.unwrap_or(""), id);
        id
    }

    pub fn check_label_name(&mut self, l: &ast::Label) -> Option<&'src str> {
        use ast::Expr as E;
        use ast::Label as L;
        match l {
            L::Empty(_) => None,
            L::Named(label) => match &*label.name_expr {
                E::Int(e) | E::Var(e) => Some(e.span.lexeme(self.source)),
                E::String(e) => Some(&self.source[(e.span.start + 1)..(e.span.end - 1)]),
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidLabel())
                            .with_primary_label(Label::Empty, l.span().wrap(self.file)),
                    );
                    None
                }
            },
        }
    }

    pub fn find_label_by_name(
        &mut self,
        label_name: Option<&str>,
        skippable: bool,
    ) -> Option<ir::LabelID> {
        self.label_scope
            .find(|&id| {
                let info = &self.get_label(id);
                match (&label_name, &info.name) {
                    (Some(query), Some(found)) => query == found,
                    (None, _) => !skippable || info.skippable,
                    _ => false,
                }
            })
            .copied()
    }
}

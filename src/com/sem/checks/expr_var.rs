use crate::com::{
    ast,
    ir::{self, TypeProvenance},
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_var(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let name = e.span.lexeme(self.source);
        let Some(&id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownVariable(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return self.check_missing();
        };

        #[allow(irrefutable_let_patterns)]
        let ir::Entity::Variable(var) = self.get_entity(id) else {
            self.reports.push(
                Report::error(Header::NotVariable(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return self.check_missing();
        };

        let loc = var.loc;
        let instantiated =
            self.instantiate_scheme(var.scheme.clone(), Some(e.span.wrap(self.file)));
        let ty = self.clone_type_repr(instantiated);
        self.set_type_span(ty, e.span);
        self.add_type_provenance(
            ty,
            TypeProvenance::VariableDefinition(loc, name.to_string()),
        );

        (ir::Expr::Var(id), ty)
    }
}

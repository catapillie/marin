use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_var_type(&mut self, t: &ast::Lexeme) -> ir::TypeID {
        let name = t.span.lexeme(self.source);
        let Some(id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownType(name.to_string()))
                    .with_primary_label(Label::Empty, t.span.wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span));
        };

        let ir::Entity::Type(info) = &self.entities[id.0] else {
            self.reports.push(
                Report::error(Header::NotType(name.to_string()))
                    .with_primary_label(Label::Empty, t.span.wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span));
        };

        use ir::TypeInfo as Info;
        let id = match info {
            Info::Type(ty) => self.clone_type_repr(*ty),
            Info::Record(info) => match &info.type_args {
                Some(type_args) => {
                    self.reports.push(
                        Report::error(Header::IncompleteType())
                            .with_primary_label(
                                Label::RecordTypeArgCount(info.name.to_string(), type_args.len()),
                                t.span.wrap(self.file),
                            )
                            .with_secondary_label(
                                Label::WithinRecordDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    self.create_fresh_type(None)
                }
                None => self.instantiate_scheme(info.scheme.clone()),
            },
            Info::Union(info) => match &info.type_args {
                Some(type_args) => {
                    self.reports.push(
                        Report::error(Header::IncompleteType())
                            .with_primary_label(
                                Label::UnionTypeArgCount(info.name.to_string(), type_args.len()),
                                t.span.wrap(self.file),
                            )
                            .with_secondary_label(
                                Label::WithinUnionDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    self.create_fresh_type(None)
                }
                None => self.instantiate_scheme(info.scheme.clone()),
            },
        };

        self.set_type_span(id, t.span);
        id
    }
}

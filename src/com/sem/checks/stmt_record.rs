use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_record(&mut self, e: &ast::Record, public: bool) -> ir::Stmt {
        // ensure the record's signature syntax is valid
        let span = e.span();
        let Some((name_span, args)) = Self::extract_simple_signature(&e.signature) else {
            self.reports.push(
                Report::error(Header::InvalidSignature())
                    .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                    .with_note(Note::RecordSyntax),
            );
            return ir::Stmt::Nothing;
        };

        let record_name = name_span.lexeme(self.source);
        let within_label = Label::WithinRecordDefinition(record_name.to_string());
        self.open_scope(false);

        // analyse and declare its type arguments if there are any
        let (arg_info, arg_ids) = if let Some(args) = args {
            let mut arg_info = Vec::new();
            let mut arg_ids = Vec::new();

            if args.is_empty() {
                self.reports.push(
                    Report::warning(Header::RecordNoArgs(record_name.to_string()))
                        .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::UseSimpleRecordSyntax(record_name.to_string())),
                );
            }

            for arg in args {
                let (arg_id, arg_name) = self.declare_type_argument(arg);
                arg_info.push(ir::RecordArgInfo { name: arg_name });
                arg_ids.push(arg_id);
            }

            (
                Some(arg_info.into_boxed_slice()),
                Some(arg_ids.into_boxed_slice()),
            )
        } else {
            (None, None)
        };

        // register the record type now
        // allows for recursion
        let record_id = self.create_entity_dummy();
        let record_type = self.create_type(ir::Type::Record(record_id, arg_ids), None);
        let record_scheme = self.generalize_type(record_type);
        let record_loc = span.wrap(self.file);
        let info = ir::Entity::Record(ir::RecordInfo {
            name: record_name.to_string(),
            type_args: arg_info,
            loc: record_loc,
            scheme: record_scheme,
            fields: Box::new([]),
        });
        *self.get_entity_mut(record_id) = info;
        self.set_entity_public(record_id, public);

        // bind it to its name now so that it can be used recursively
        self.scope.insert(record_name, record_id);

        // check fields
        let mut fields = Vec::new();
        for (name, ty) in &e.fields {
            let field_ty = self.check_type(ty);

            use ast::Expr as E;
            let E::Var(field_name_span) = name else {
                self.reports.push(
                    Report::error(Header::InvalidField())
                        .with_primary_label(Label::Empty, name.span().wrap(self.file))
                        .with_note(Note::RecordFieldSyntax),
                );
                continue;
            };

            let field_name = field_name_span.span.lexeme(self.source);
            fields.push(ir::RecordFieldInfo {
                name: field_name.to_string(),
                ty: field_ty,
                loc: field_name_span.span.wrap(self.file),
            });
        }

        // close scope, but export the record's name binding
        self.close_scope();
        self.scope.insert(record_name, record_id);

        let info = self.get_record_info_mut(record_id);
        info.fields = fields.into();

        // done
        ir::Stmt::Nothing
    }

    pub fn is_record_admissible(info: &ir::RecordInfo, names: &[&str]) -> bool {
        for name in names {
            let found_field = info.fields.iter().any(|rec| &rec.name == name);
            if !found_field {
                return false;
            }
        }

        true
    }
}

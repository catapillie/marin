use crate::com::{
    ast, ir,
    loc::Span,
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
        let record_id = self.entities.next_record_id();
        let record_type = self.create_type(ir::Type::Record(record_id, arg_ids), None);
        let record_scheme = self.generalize_type(record_type);
        let record_loc = span.wrap(self.file);
        self.entities.create_record(ir::RecordInfo {
            name: record_name.to_string(),
            type_args: arg_info,
            loc: record_loc,
            scheme: record_scheme,
            fields: Box::new([]),
        });
        self.set_entity_public(record_id.wrap(), public);

        // bind it to its name now so that it can be used recursively
        self.scope.insert(record_name, record_id.wrap());

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
        self.scope.insert(record_name, record_id.wrap());

        let info = self.entities.get_record_info_mut(record_id);
        info.fields = fields.into();

        // done
        ir::Stmt::Nothing
    }

    pub fn get_admissible_records(
        &mut self,
        field_names: &[&str],
    ) -> Vec<(ir::RecordID, &ir::RecordInfo)> {
        self.entities
            .records
            .iter()
            .enumerate()
            .filter_map(
                |(i, info)| match Self::is_record_admissible(info, field_names) {
                    true => Some((ir::RecordID(i), info)),
                    false => todo!(),
                },
            )
            .collect::<Vec<_>>()
    }

    fn is_record_admissible(info: &ir::RecordInfo, names: &[&str]) -> bool {
        for name in names {
            let found_field = info.fields.iter().any(|rec| &rec.name == name);
            if !found_field {
                return false;
            }
        }

        true
    }

    /// Produces an error report if the provided arguments don't match the record type's signature
    pub fn create_record_type(
        &mut self,
        record_id: ir::RecordID,
        args: Option<Box<[ir::TypeID]>>,
        span: Span,
    ) -> Option<ir::TypeID> {
        let info = self.entities.get_record_info(record_id);
        let Some(args) = args else {
            match &info.type_args {
                None => {
                    return Some(self.create_type(ir::Type::Record(record_id, None), Some(span)))
                }
                Some(type_args) => {
                    let arity = type_args.len();
                    self.reports.push(
                        Report::error(Header::RecordArgMismatch(info.name.to_string()))
                            .with_primary_label(
                                Label::RecordTypeArgCount(info.name.to_string(), arity),
                                span.wrap(self.file),
                            )
                            .with_secondary_label(
                                Label::RecordDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    return None;
                }
            }
        };

        let info = self.entities.get_record_info(record_id);
        let Some(type_args) = &info.type_args else {
            self.reports.push(
                Report::error(Header::RecordArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::RecordTypeNoArgs(info.name.to_string()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(info.name.to_string()), info.loc),
            );
            return None;
        };

        let arity = type_args.len();
        let record_ty = self.instantiate_scheme(info.scheme.clone(), None);
        let record_ty = self.clone_type_repr(record_ty);
        self.set_type_span(record_ty, span);

        let info = self.entities.get_record_info(record_id);
        if arity == args.len() {
            let ty = self.create_type(ir::Type::Record(record_id, Some(args)), Some(span));
            self.unify(ty, record_ty, &[]);
        } else {
            self.reports.push(
                Report::error(Header::RecordArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::RecordTypeArgCount(info.name.to_string(), arity),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(info.name.to_string()), info.loc),
            );
            return None;
        }

        Some(record_ty)
    }
}

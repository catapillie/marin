use std::collections::HashMap;

use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    sem::provenance::Provenance,
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_record_value(&mut self, e: &ast::RecordValue) -> ir::CheckedExpr {
        let mut fields = HashMap::new();
        for (name, expr) in &e.fields {
            // find the field's name
            use ast::Expr as E;
            let field_name_span = match name {
                E::Var(s) => Some(s.span),
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidField())
                            .with_primary_label(Label::Empty, name.span().wrap(self.file))
                            .with_note(Note::RecordFieldSyntax),
                    );
                    None
                }
            };

            // ensure there is an expression on the rhs
            // otherwise push an error and return a dummy expression
            let (value, field_ty) = match expr {
                Some(expr) => self.check_expression(expr),
                _ => {
                    self.reports.push(
                        Report::error(Header::RequiredFieldValue())
                            .with_primary_label(Label::Empty, name.span().wrap(self.file))
                            .with_note(Note::FieldValueSyntax),
                    );
                    self.check_missing()
                }
            };

            // if there's at least a name, add it to the map of fields (by name)
            if let Some(field_name_span) = field_name_span {
                let field_name = field_name_span.lexeme(self.source);
                fields.insert(field_name, (value, field_ty));
            }
        }

        use ir::Entity as Ent;
        let field_names = fields.keys().copied().collect::<Vec<_>>();
        let mut record_types = self
            .entities
            .iter()
            .enumerate()
            .filter_map(|(i, ent)| match ent {
                Ent::Record(info) => Some((ir::EntityID(i), info)),
                _ => None,
            })
            .filter(|(_, info)| Self::is_record_admissible(info, &field_names))
            .collect::<Vec<_>>();

        if record_types.is_empty() {
            self.reports.push(
                Report::error(Header::NoAdmissibleRecords()).with_primary_label(
                    Label::NoAdmissibleRecord(fields.len()),
                    e.span().wrap(self.file),
                ),
            );
            return self.check_missing();
        }

        if record_types.len() > 1 {
            self.reports.push(
                Report::error(Header::AmbiguousRecord())
                    .with_primary_label(Label::Empty, e.span().wrap(self.file)),
            );
            return self.check_missing();
        }

        let (record_id, info) = record_types.pop().unwrap();
        let record_name = info.name.clone();
        let record_loc = info.loc;

        let scheme = info.scheme.clone();
        let sub = self.build_type_substitution(scheme.forall);

        let record_value_type = self.apply_type_substitution(scheme.uninstantiated, &sub);
        let record_value_type = self.clone_type_repr(record_value_type);
        self.set_type_span(record_value_type, e.span());

        // check that all fields are actually set
        let mut missing_fields = Vec::new();
        let mut set_fields = Vec::new();
        let info = self.get_record_info(record_id);
        for (i, field_info) in info.fields.clone().iter().enumerate() {
            let Some((field_value, field_value_ty)) = fields.remove(field_info.name.as_str())
            else {
                missing_fields.push(i);
                continue;
            };
            set_fields.push(field_value);

            let provenances = &[Provenance::RecordFieldTypes(
                record_name.clone(),
                record_loc,
            )];

            let field_ty = self.apply_type_substitution(field_info.ty, &sub);
            let field_ty = self.clone_type_repr(field_ty);
            self.set_type_loc(field_ty, field_info.loc);
            self.unify(field_value_ty, field_ty, provenances);
        }

        let info = self.get_record_info(record_id);
        if !missing_fields.is_empty() {
            let missing_field_names = missing_fields
                .into_iter()
                .map(|id| info.fields[id].name.clone())
                .collect();
            self.reports.push(
                Report::error(Header::UninitializedFields(record_name.clone()))
                    .with_primary_label(
                        Label::MissingFields(missing_field_names, record_name.clone()),
                        e.span().wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(record_name), record_loc),
            );
        }

        (ir::Expr::Record(set_fields.into()), record_value_type)
    }
}

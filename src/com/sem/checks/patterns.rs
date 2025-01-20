use super::path::PathQuery as Q;
use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Note, Report},
    sem::provenance::Provenance,
    Checker,
};

use std::collections::HashMap;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_pattern(&mut self, e: &ast::Expr) -> ast::Pattern {
        use ast::Expr as E;
        use ast::Pattern as P;
        match e {
            E::Missing(e) => P::Missing(e.span),
            E::Var(e) => P::Binding(e.span),
            E::Int(e) => P::Int(e.span),
            E::Float(e) => P::Float(e.span),
            E::String(e) => P::String(e.span),
            E::True(e) => P::True(e.span),
            E::False(e) => P::False(e.span),
            E::Tuple(e) if e.items.len() == 1 => self.check_pattern(&e.items[0]),
            E::Tuple(e) => P::Tuple(
                e.left_paren,
                e.right_paren,
                e.items
                    .iter()
                    .map(|item| self.check_pattern(item))
                    .collect(),
            ),
            E::Call(e) => P::Call(
                e.left_paren,
                e.right_paren,
                e.callee.clone(),
                e.args.iter().map(|arg| self.check_pattern(arg)).collect(),
            ),
            E::Access(_) => P::Access(Box::new(e.clone())),
            E::RecordValue(e) => {
                let mut fields = Vec::new();
                for (name, expr) in &e.fields {
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

                    let pat = expr.as_ref().map(|e| self.check_pattern(e));
                    fields.push((field_name_span, pat));
                }
                P::Record(e.left_brace, e.right_brace, fields.into())
            }
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidPattern())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                P::Missing(e.span())
            }
        }
    }

    pub fn declare_pattern(&mut self, p: &ast::Pattern) -> (ir::Pattern, ir::TypeID) {
        use ast::Pattern as P;
        use ir::Pattern as I;
        let span = p.span();
        match p {
            P::Missing(_) => self.declare_missing_pattern(),
            P::Binding(_) => {
                let name = span.lexeme(self.source);
                let ty = self.create_fresh_type(Some(span));
                let id = self.create_variable_mono(name, ty, span);
                (I::Binding(id), ty)
            }
            P::Int(_) => (
                self.read_source_int(span).map(I::Int).unwrap_or(I::Missing),
                self.create_type(ir::Type::Int, Some(span)),
            ),
            P::Float(_) => (
                self.read_source_float(span)
                    .map(I::Float)
                    .unwrap_or(I::Missing),
                self.create_type(ir::Type::Float, Some(span)),
            ),
            P::String(_) => (
                I::String(self.read_source_string(span).to_string()),
                self.create_type(ir::Type::String, Some(span)),
            ),
            P::True(_) => (I::Bool(true), self.create_type(ir::Type::Bool, Some(span))),
            P::False(_) => (I::Bool(false), self.create_type(ir::Type::Bool, Some(span))),
            P::Tuple(_, _, items) => {
                let (items, item_types): (Vec<_>, Vec<_>) =
                    items.iter().map(|item| self.declare_pattern(item)).unzip();
                (
                    I::Tuple(items.into()),
                    self.create_type(ir::Type::Tuple(item_types.into()), Some(span)),
                )
            }
            P::Call(_, _, e, args) => self.declare_call_pattern(e, args, span),
            P::Access(e) => self.declare_access_pattern(e, span),
            P::Record(_, _, fields) => self.declare_record_pattern(fields, span),
        }
    }

    fn declare_call_pattern(
        &mut self,
        e: &ast::Expr,
        args: &[ast::Pattern],
        span: Span,
    ) -> (ir::Pattern, ir::TypeID) {
        let q = self.check_path(e);
        let (args, arg_types): (Vec<_>, Vec<_>) =
            args.iter().map(|item| self.declare_pattern(item)).unzip();

        use ir::Pattern as I;
        match q {
            Q::Variant(id, tag) => {
                let (info, variant) = self.get_union_variant_info(id, tag);

                let Some(variant_args) = &variant.type_args else {
                    self.reports.push(
                        Report::error(Header::IncorrectVariantArgs(variant.name.to_string()))
                            .with_primary_label(Label::Empty, span.wrap(self.file))
                            .with_secondary_label(
                                Label::VariantDefinition(variant.name.to_string()),
                                variant.loc,
                            )
                            .with_secondary_label(
                                Label::UnionDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    return self.declare_missing_pattern();
                };

                let mut variant_args = variant_args.clone();
                if variant_args.len() != args.len() {
                    self.reports.push(
                        Report::error(Header::IncorrectVariantArgCount(
                            variant.name.to_string(),
                            variant_args.len(),
                            args.len(),
                        ))
                        .with_primary_label(Label::Empty, span.wrap(self.file))
                        .with_secondary_label(
                            Label::VariantDefinition(variant.name.to_string()),
                            variant.loc,
                        )
                        .with_secondary_label(
                            Label::UnionDefinition(info.name.to_string()),
                            info.loc,
                        ),
                    );
                }

                let (info, variant) = self.get_union_variant_info(id, tag);
                let union_type = info.scheme.uninstantiated;

                let sub = self.build_type_substitution(variant.scheme.forall.clone());
                let union_type = self.apply_type_substitution(union_type, &sub);
                for (i, variant_arg) in variant_args.iter_mut().enumerate() {
                    *variant_arg = self.apply_type_substitution(*variant_arg, &sub);
                    if let Some(arg) = arg_types.get(i) {
                        self.unify(*arg, *variant_arg, &[]);
                    }
                }

                let ty = self.clone_type_repr(union_type);
                self.set_type_span(ty, span);
                (I::Variant(id, tag, Some(args.into())), ty)
            }
            Q::Missing => self.declare_missing_pattern(),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidPattern())
                        .with_primary_label(Label::NotAPattern, span.wrap(self.file)),
                );
                self.declare_missing_pattern()
            }
        }
    }

    fn declare_access_pattern(&mut self, e: &ast::Expr, span: Span) -> (ir::Pattern, ir::TypeID) {
        use ir::Pattern as I;
        match self.check_path(e) {
            Q::Variant(id, tag) => {
                let (info, variant) = self.get_union_variant_info(id, tag);

                if let Some(variant_args) = &variant.type_args {
                    self.reports.push(
                        Report::error(Header::IncompleteVariant(variant.name.to_string()))
                            .with_primary_label(Label::Empty, span.wrap(self.file))
                            .with_secondary_label(
                                Label::VariantArgCount(
                                    variant.name.to_string(),
                                    variant_args.len(),
                                ),
                                variant.loc,
                            )
                            .with_secondary_label(
                                Label::UnionDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    return self.declare_missing_pattern();
                };

                let ty = self.instantiate_scheme(info.scheme.clone());
                self.set_type_span(ty, span);
                (I::Variant(id, tag, None), ty)
            }
            Q::Missing => self.declare_missing_pattern(),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidPattern())
                        .with_primary_label(Label::NotAPattern, span.wrap(self.file)),
                );
                self.declare_missing_pattern()
            }
        }
    }

    fn declare_record_pattern(
        &mut self,
        field_pats: &[(Option<Span>, Option<ast::Pattern>)],
        span: Span,
    ) -> (ir::Pattern, ir::TypeID) {
        let mut fields = HashMap::new();
        for (name, pat) in field_pats {
            let Some(name_span) = name else {
                continue;
            };

            let field_name = name_span.lexeme(self.source);
            let field_pattern = match pat {
                Some(pat) => pat,
                None => &ast::Pattern::Binding(*name_span),
            };

            let pat = self.declare_pattern(field_pattern);
            fields.insert(field_name, pat);
        }

        use ir::Entity as Ent;
        use ir::TypeInfo as T;
        let field_names = fields.keys().copied().collect::<Vec<_>>();
        let mut record_types = self
            .entities
            .iter()
            .enumerate()
            .filter_map(|(i, ent)| match ent {
                Ent::Type(T::Record(info)) => Some((ir::EntityID(i), info)),
                _ => None,
            })
            .filter(|(_, info)| Self::is_record_admissible(info, &field_names))
            .collect::<Vec<_>>();

        if record_types.is_empty() {
            self.reports.push(
                Report::error(Header::NoAdmissibleRecords()).with_primary_label(
                    Label::NoAdmissibleRecord(fields.len()),
                    span.wrap(self.file),
                ),
            );
            return self.declare_missing_pattern();
        }

        if record_types.len() > 1 {
            self.reports.push(
                Report::error(Header::AmbiguousRecord())
                    .with_primary_label(Label::Empty, span.wrap(self.file)),
            );
            return self.declare_missing_pattern();
        }

        let (record_id, info) = record_types.pop().unwrap();
        let record_name = info.name.clone();
        let record_loc = info.loc;

        let scheme = info.scheme.clone();
        let sub = self.build_type_substitution(scheme.forall);

        let record_value_type = self.apply_type_substitution(scheme.uninstantiated, &sub);
        let record_value_type = self.clone_type_repr(record_value_type);
        self.set_type_span(record_value_type, span);

        // check that all fields are actually set
        let mut missing_fields = Vec::new();
        let mut set_fields = Vec::new();
        let info = self.get_record_info(record_id);
        for (i, field_info) in info.fields.clone().iter().enumerate() {
            let Some((field_pat, field_pat_ty)) = fields.remove(field_info.name.as_str()) else {
                missing_fields.push(i);
                continue;
            };
            set_fields.push(field_pat);

            let provenances = &[Provenance::RecordFieldTypes(
                record_name.clone(),
                record_loc,
            )];

            let field_ty = self.apply_type_substitution(field_info.ty, &sub);
            let field_ty = self.clone_type_repr(field_ty);
            self.set_type_loc(field_ty, field_info.loc);
            self.unify(field_pat_ty, field_ty, provenances);
        }

        let info = self.get_record_info(record_id);
        if !missing_fields.is_empty() {
            let missing_field_names = missing_fields
                .into_iter()
                .map(|id| info.fields[id].name.clone())
                .collect();
            self.reports.push(
                Report::error(Header::UnmatchedFields(record_name.clone()))
                    .with_primary_label(
                        Label::MissingFields(missing_field_names, record_name.clone()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(record_name), record_loc),
            );
        }

        (
            ir::Pattern::Record(record_id, set_fields.into()),
            record_value_type,
        )
    }

    fn declare_missing_pattern(&mut self) -> (ir::Pattern, ir::TypeID) {
        (ir::Pattern::Missing, self.create_fresh_type(None))
    }
}

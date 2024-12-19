use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_union(&mut self, e: &ast::Union) -> ir::Stmt {
        use ast::Expr as E;

        let span = e.span();
        let Some((name_span, args)) = (match &*e.signature {
            E::Var(e) => Some((e.span, None)),
            E::Call(call) => match &*call.callee {
                E::Var(e) => Some((e.span, Some(&*call.args))),
                _ => None,
            },
            _ => None,
        }) else {
            self.reports.push(
                Report::error(Header::InvalidSignature())
                    .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                    .with_note(Note::UnionSyntax),
            );
            return ir::Stmt::Nothing;
        };

        let name = name_span.lexeme(self.source);
        let within_label = Label::WithinUnionDefinition(name.to_string());
        self.open_scope(false);

        let (arg_info, arg_ids) = if let Some(args) = args {
            let mut arg_info = Vec::new();
            let mut arg_ids = Vec::new();

            if args.is_empty() {
                self.reports.push(
                    Report::error(Header::UnionNoArgs(name.to_string()))
                        .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::UseSimpleUnionSyntax(name.to_string())),
                );
            }

            for arg in args {
                let (arg_id, arg_name) = match arg {
                    E::Var(e) => {
                        let arg_span = e.span;
                        let arg_name = arg_span.lexeme(self.source);
                        let arg_id = self.create_fresh_type(Some(e.span));
                        self.create_user_type(arg_name, ir::TypeInfo::Type(arg_id));
                        (arg_id, Some(arg_name.to_string()))
                    }
                    _ => {
                        self.reports.push(
                            Report::error(Header::InvalidTypeArg())
                                .with_primary_label(Label::Empty, arg.span().wrap(self.file)),
                        );
                        (self.create_fresh_type(Some(arg.span())), None)
                    }
                };
                arg_info.push(ir::UnionTypeArg { name: arg_name });
                arg_ids.push(arg_id);
            }

            (
                Some(arg_info.into_boxed_slice()),
                Some(arg_ids.into_boxed_slice()),
            )
        } else {
            (None, None)
        };

        let union_id = ir::EntityID(self.entities.len());
        let union_type = self.create_type(ir::Type::Union(union_id, arg_ids), None);
        let union_scheme = self.generalize_type(union_type);
        let info = ir::TypeInfo::Union(ir::UnionInfo {
            name: name.to_string(),
            type_args: arg_info,
            loc: e.signature.span().wrap(self.file),
            scheme: union_scheme,
        });
        self.entities.push(ir::Entity::Type(info));
        self.scope.insert(name, union_id);

        for variant in &e.variants {
            let Some((variant_name_span, variant_args)) = (match variant {
                E::Var(e) => Some((e.span, None)),
                E::Call(call) => match &*call.callee {
                    E::Var(e) => Some((e.span, Some(&*call.args))),
                    _ => None,
                },
                _ => None,
            }) else {
                self.reports.push(
                    Report::error(Header::InvalidSignature())
                        .with_primary_label(Label::Empty, variant.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::VariantSyntax),
                );
                continue;
            };

            let variant_name = variant_name_span.lexeme(self.source);
            if let Some(variant_args) = variant_args {
                if variant_args.is_empty() {
                    self.reports.push(
                        Report::error(Header::UnionVariantNoArgs(variant_name.to_string()))
                            .with_primary_label(Label::Empty, variant.span().wrap(self.file))
                            .with_secondary_label(within_label.clone(), span.wrap(self.file))
                            .with_note(Note::UseConstantUnionSyntax(variant_name.to_string())),
                    );
                }

                for arg in variant_args {
                    self.check_type(arg);
                }
            }
        }

        self.close_scope();
        self.scope.insert(name, union_id);

        ir::Stmt::Nothing
    }
}

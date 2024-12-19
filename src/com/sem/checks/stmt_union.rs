use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    fn extract_simple_signature(e: &ast::Expr) -> Option<(Span, Option<&[ast::Expr]>)> {
        use ast::Expr as E;
        match e {
            E::Var(e) => Some((e.span, None)),
            E::Call(call) => match &*call.callee {
                E::Var(e) => Some((e.span, Some(&*call.args))),
                _ => None,
            },
            _ => None,
        }
    }

    fn declare_type_argument(&mut self, e: &ast::Expr) -> (ir::TypeID, Option<String>) {
        use ast::Expr as E;
        match e {
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
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                (self.create_fresh_type(Some(e.span())), None)
            }
        }
    }

    pub fn check_union(&mut self, e: &ast::Union) -> ir::Stmt {
        // ensure the union's signature syntax is valid
        let span = e.span();
        let Some((name_span, args)) = Self::extract_simple_signature(&e.signature) else {
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

        // analyse and declare its type arguments if there are any
        let (arg_info, arg_ids) = if let Some(args) = args {
            let mut arg_info = Vec::new();
            let mut arg_ids = Vec::new();

            if args.is_empty() {
                self.reports.push(
                    Report::warning(Header::UnionNoArgs(name.to_string()))
                        .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::UseSimpleUnionSyntax(name.to_string())),
                );
            }

            for arg in args {
                let (arg_id, arg_name) = self.declare_type_argument(arg);
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

        // register the union type now
        // allows for recursion
        let union_id = self.next_entity_id();
        let union_type = self.create_type(ir::Type::Union(union_id, arg_ids), None);
        let union_scheme = self.generalize_type(union_type);
        let info = ir::TypeInfo::Union(ir::UnionInfo {
            name: name.to_string(),
            type_args: arg_info,
            loc: e.signature.span().wrap(self.file),
            scheme: union_scheme,
        });
        self.entities.push(ir::Entity::Type(info));

        // bind it to its name now so that it can be used recursively
        self.scope.insert(name, union_id);

        // check variants
        for variant in &e.variants {
            // ensure the variant's signature syntax is valid
            let Some((variant_name_span, variant_args)) = Self::extract_simple_signature(variant)
            else {
                self.reports.push(
                    Report::error(Header::InvalidSignature())
                        .with_primary_label(Label::Empty, variant.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::VariantSyntax),
                );
                continue;
            };

            // check the variant arguments if any
            let variant_name = variant_name_span.lexeme(self.source);
            if let Some(variant_args) = variant_args {
                if variant_args.is_empty() {
                    self.reports.push(
                        Report::warning(Header::UnionVariantNoArgs(variant_name.to_string()))
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

        // close scope, but export the union's name binding
        self.close_scope();
        self.scope.insert(name, union_id);

        // done
        ir::Stmt::Nothing
    }
}

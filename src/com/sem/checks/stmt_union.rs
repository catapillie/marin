use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_union(&mut self, e: &ast::Union, public: bool) -> ir::Stmt {
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

        let union_name = name_span.lexeme(self.source);
        let within_label = Label::WithinUnionDefinition(union_name.to_string());
        self.open_scope(false);

        // analyse and declare its type arguments if there are any
        let (arg_info, arg_ids) = if let Some(args) = args {
            let mut arg_info = Vec::new();
            let mut arg_ids = Vec::new();

            if args.is_empty() {
                self.reports.push(
                    Report::warning(Header::UnionNoArgs(union_name.to_string()))
                        .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::UseSimpleUnionSyntax(union_name.to_string())),
                );
            }

            for arg in args {
                let (arg_id, arg_name) = self.declare_type_argument(arg);
                arg_info.push(ir::UnionArgInfo { name: arg_name });
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
        let union_id = self.entities.next_union_id();
        let union_type = self.create_type(ir::Type::Union(union_id, arg_ids), None);
        let union_scheme = self.generalize_type(union_type);
        let union_loc = span.wrap(self.file);
        self.entities.create_union(ir::UnionInfo {
            name: union_name.to_string(),
            type_args: arg_info,
            loc: union_loc,
            scheme: union_scheme,
            variants: Box::new([]),
        });
        self.set_entity_public(union_id.wrap(), public);

        // bind it to its name now so that it can be used recursively
        self.scope.insert(union_name, union_id.wrap());

        // check variants
        let mut variants = Vec::new();
        for (tag, variant) in e.variants.iter().enumerate() {
            // ensure the variant's signature syntax is valid
            let Some((variant_name_span, variant_args)) = Self::extract_simple_signature(variant)
            else {
                self.reports.push(
                    Report::error(Header::InvalidSignature())
                        .with_primary_label(Label::Empty, variant.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::UnionVariantSyntax),
                );
                continue;
            };

            // check the variant arguments if any
            let variant_name = variant_name_span.lexeme(self.source);
            let variant_type_args = if let Some(variant_args) = variant_args {
                if variant_args.is_empty() {
                    self.reports.push(
                        Report::warning(Header::UnionVariantNoArgs(variant_name.to_string()))
                            .with_primary_label(Label::Empty, variant.span().wrap(self.file))
                            .with_secondary_label(within_label.clone(), span.wrap(self.file))
                            .with_note(Note::UseConstantUnionSyntax(variant_name.to_string())),
                    );
                }

                let type_args: Box<_> = variant_args
                    .iter()
                    .map(|arg| self.check_type(arg))
                    .collect();
                Some(type_args)
            } else {
                None
            };

            let arity = variant_type_args.as_ref().map(|args| args.len());

            let variant_full_name = format!("{union_name}.{variant_name}");
            let variant_expr = self.build_variant_expr(tag, arity, variant_full_name);
            let variant_type = match variant_type_args.clone() {
                Some(args) => {
                    self.create_type(ir::Type::Lambda(args, union_type), Some(variant_name_span))
                }
                None => union_type,
            };

            let variant_loc = variant.span().wrap(self.file);
            let scheme = self.generalize_type(variant_type);

            variants.push(ir::VariantInfo {
                name: variant_name.to_string(),
                loc: variant_loc,
                expr: variant_expr,
                scheme,
                type_args: variant_type_args,
            });
        }

        // close scope, but export the union's name binding
        self.close_scope();
        self.scope.insert(union_name, union_id.wrap());

        let info = self.entities.get_union_info_mut(union_id);
        info.variants = variants.into();

        // done
        ir::Stmt::Nothing
    }

    fn build_variant_expr(&mut self, tag: usize, arity: Option<usize>, name: String) -> ir::Expr {
        self.open_scope(true);

        let expr = match arity {
            None => ir::Expr::Variant { tag, items: None },
            Some(arity) => {
                let arg_ids = (0..arity)
                    .map(|_| ir::VariableID::dummy())
                    .collect::<Vec<_>>();
                let arg_patterns = arg_ids.iter().map(|id| ir::Pattern::Binding(*id)).collect();
                let arg_exprs = arg_ids.iter().map(|id| ir::Expr::Var { id: *id }).collect();

                ir::Expr::Fun { name, recursive_binding: None, signature: Box::new(ir::Signature::Args { args: arg_patterns, next: Box::new(ir::Signature::Done) }), expr: Box::new(ir::Expr::Variant { tag, items: Some(arg_exprs) }) }
            }
        };

        self.close_scope();
        expr
    }

    /// Produces an error report if the provided arguments don't match the union type's signature
    pub fn create_union_type(
        &mut self,
        union_id: ir::UnionID,
        args: Option<Box<[ir::TypeID]>>,
        span: Span,
    ) -> Option<ir::TypeID> {
        let info = self.entities.get_union_info(union_id);
        let Some(args) = args else {
            match &info.type_args {
                None => return Some(self.create_type(ir::Type::Union(union_id, None), Some(span))),
                Some(type_args) => {
                    let arity = type_args.len();
                    self.reports.push(
                        Report::error(Header::UnionArgMismatch(info.name.to_string()))
                            .with_primary_label(
                                Label::UnionTypeArgCount(info.name.to_string(), arity),
                                span.wrap(self.file),
                            )
                            .with_secondary_label(
                                Label::UnionDefinition(info.name.to_string()),
                                info.loc,
                            ),
                    );
                    return None;
                }
            }
        };

        let info = self.entities.get_union_info(union_id);
        let Some(type_args) = &info.type_args else {
            self.reports.push(
                Report::error(Header::UnionArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::UnionTypeNoArgs(info.name.to_string()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::UnionDefinition(info.name.to_string()), info.loc),
            );
            return None;
        };

        let arity = type_args.len();
        let union_ty = self.instantiate_scheme(info.scheme.clone(), None);
        let union_ty = self.clone_type_repr(union_ty);
        self.set_type_span(union_ty, span);

        let info = self.entities.get_union_info(union_id);
        if arity == args.len() {
            let ty = self.create_type(ir::Type::Union(union_id, Some(args)), Some(span));
            self.unify(ty, union_ty, &[]);
        } else {
            self.reports.push(
                Report::error(Header::UnionArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::UnionTypeArgCount(info.name.to_string(), arity),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::UnionDefinition(info.name.to_string()), info.loc),
            );
            return None;
        }

        Some(union_ty)
    }
}

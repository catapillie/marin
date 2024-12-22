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
        let union_id = self.next_entity_id();
        let union_type = self.create_type(ir::Type::Union(union_id, arg_ids), None);
        let union_scheme = self.generalize_type(union_type);
        let union_loc = span.wrap(self.file);
        let info = ir::TypeInfo::Union(ir::UnionInfo {
            name: union_name.to_string(),
            type_args: arg_info,
            loc: union_loc,
            scheme: union_scheme,
            variants: Box::new([]),
        });
        self.entities.push(ir::Entity::Type(info));

        // bind it to its name now so that it can be used recursively
        self.scope.insert(union_name, union_id);

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
                        .with_note(Note::VariantSyntax),
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

            let variant_expr = self.build_variant_expr(tag, arity);
            let variant_type = match variant_type_args.clone() {
                Some(args) => {
                    self.create_type(ir::Type::Lambda(args, union_type), Some(variant_name_span))
                }
                None => union_type,
            };

            let variant_loc = variant.span().wrap(self.file);

            variants.push(ir::VariantInfo {
                name: variant_name.to_string(),
                loc: variant_loc,
                expr: variant_expr,
                scheme: self.generalize_type(variant_type),
                type_args: variant_type_args,
            });
        }

        // close scope, but export the union's name binding
        self.close_scope();
        self.scope.insert(union_name, union_id);

        let ir::Entity::Type(ir::TypeInfo::Union(info)) = self.get_entity_mut(union_id) else {
            unreachable!()
        };
        info.variants = variants.into();

        // done
        ir::Stmt::Nothing
    }

    fn build_variant_expr(&mut self, tag: usize, arity: Option<usize>) -> ir::Expr {
        self.open_scope(true);

        let expr = match arity {
            None => ir::Expr::Variant(tag, None),
            Some(arity) => {
                let arg_ids = (0..arity)
                    .map(|_| self.create_entity_dummy())
                    .collect::<Vec<_>>();
                let arg_patterns = arg_ids.iter().map(|id| ir::Pattern::Binding(*id)).collect();
                let arg_exprs = arg_ids.iter().map(|id| ir::Expr::Var(*id)).collect();

                ir::Expr::Fun(
                    None,
                    Box::new(ir::Signature::Args(
                        arg_patterns,
                        Box::new(ir::Signature::Done),
                    )),
                    Box::new(ir::Expr::Variant(tag, Some(arg_exprs))),
                )
            }
        };

        self.close_scope();
        expr
    }
}

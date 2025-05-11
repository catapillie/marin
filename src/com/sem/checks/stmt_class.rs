use crate::com::{
    Checker, ast, ir,
    loc::Span,
    reporting::{Header, Label, Note, Report},
    sem::checker::checker_print,
};
use colored::Colorize;
use either::Either;

impl Checker<'_, '_> {
    pub fn check_pattern_or_type_signature(
        &mut self,
        e: &ast::Expr,
    ) -> Either<ast::Pattern, (ast::TypeSignature, Option<Span>)> {
        use ast::Expr as E;
        match e {
            E::Call(..) => Either::Right(self.check_type_signature(e)),
            _ => Either::Left(self.check_pattern(e)),
        }
    }

    pub fn check_class(&mut self, e: &ast::Class, public: bool) -> ir::Stmt {
        let span = e.span();
        let Some((class_name_span, args)) = Self::extract_simple_signature_with_args(&e.signature)
        else {
            self.reports.push(
                Report::error(Header::InvalidSignature())
                    .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                    .with_note(Note::ClassSyntax),
            );
            return ir::Stmt::Nothing;
        };

        let class_name = class_name_span.lexeme(self.source);
        let within_label = Label::WithinClassDefinition(class_name.to_string());

        self.open_scope(false);

        if args.is_empty() {
            self.reports.push(
                Report::error(Header::ClassNoArgs(class_name.to_string()))
                    .with_primary_label(Label::Empty, e.signature.span().wrap(self.file))
                    .with_secondary_label(within_label.clone(), span.wrap(self.file))
                    .with_note(Note::ClassSyntax),
            );
        }

        let mut arg_ids = Vec::new();
        for arg in args {
            let (arg_id, _) = self.declare_type_argument(arg);
            arg_ids.push(arg_id);
        }

        let mut associated_arg_ids = Vec::new();
        if let Some(args) = &e.associated {
            for arg in args {
                let (arg_id, _) = self.declare_type_argument(arg);
                associated_arg_ids.push(arg_id);
            }
        }

        let arity = (arg_ids.len(), associated_arg_ids.len());

        let class_id = self.entities.create_class(ir::ClassInfo {
            name: class_name.to_string(),
            loc: span.wrap(self.file),
            items: Default::default(),
            arity,
        });
        self.set_entity_public(class_id.wrap(), public);

        let mut items = Vec::new();
        let mut constraint = ir::Constraint {
            id: class_id,
            loc: span.wrap(self.file),
            class_args: arg_ids.into(),
            associated_args: associated_arg_ids.into(),
        };

        use ast::ClassItem as K;
        use ast::Pattern as P;
        for (kind, lhs, rhs) in &e.items {
            let item_span = Span::combine(lhs.span(), rhs.span());
            let (item_name, item_name_span, item_type) = match self
                .check_pattern_or_type_signature(lhs)
            {
                Either::Left(pattern) => {
                    let P::Var(item_name_span) = pattern else {
                        self.reports.push(
                            Report::error(Header::InvalidPattern())
                                .with_primary_label(Label::Empty, lhs.span().wrap(self.file))
                                .with_secondary_label(within_label.clone(), span.wrap(self.file))
                                .with_note(Note::ClassConstantItemSyntax),
                        );
                        continue;
                    };

                    if !matches!(kind, K::Constant | K::Unknown) {
                        self.reports.push(
                            Report::error(Header::InvalidTypeAnnotation())
                                .with_primary_label(
                                    Label::IncorrectClassConstantItemSyntax,
                                    item_span.wrap(self.file),
                                )
                                .with_secondary_label(within_label.clone(), span.wrap(self.file))
                                .with_note(Note::ClassConstantItemSyntax),
                        );
                    }

                    let item_name = item_name_span.lexeme(self.source);
                    let item_type = self.check_type(rhs);

                    (item_name, item_name_span, item_type)
                }
                Either::Right((signature, name_span)) => {
                    let Some(item_name_span) = name_span else {
                        self.reports.push(
                            Report::error(Header::InvalidSignature())
                                .with_primary_label(Label::Empty, lhs.span().wrap(self.file))
                                .with_secondary_label(within_label.clone(), span.wrap(self.file))
                                .with_note(Note::ClassFunctionItemSyntax),
                        );
                        continue;
                    };

                    if !matches!(kind, K::Function | K::Unknown) {
                        self.reports.push(
                            Report::error(Header::InvalidTypeAnnotation())
                                .with_primary_label(
                                    Label::IncorrectClassFunctionItemSyntax,
                                    item_span.wrap(self.file),
                                )
                                .with_secondary_label(within_label.clone(), span.wrap(self.file))
                                .with_note(Note::ClassFunctionItemSyntax),
                        );
                    }

                    let item_name = item_name_span.lexeme(self.source);
                    let (item_type, sig_ret_type) = self.declare_type_signature(&signature);
                    let ret_type = self.check_type(rhs);
                    self.unify(ret_type, sig_ret_type, &[]);

                    (item_name, item_name_span, item_type)
                }
            };

            let mut scheme = self.generalize_type(item_type);

            constraint.loc = item_name_span.wrap(self.file);
            self.add_class_constraint(&mut scheme, constraint.clone());

            items.push(ir::ClassItemInfo {
                name: item_name.to_string(),
                loc: item_span.wrap(self.file),
                scheme,
            });
        }

        let info = self.entities.get_class_info_mut(class_id);
        info.items = items.into();

        self.close_scope();
        self.scope.insert(class_name, class_id.wrap());

        let items = self
            .entities
            .get_class_info(class_id)
            .items
            .iter()
            .map(|item| (item.name.clone(), item.scheme.clone()))
            .collect::<Vec<_>>();
        checker_print!(self, "{} {}", "class".bold(), class_name);
        for (item_name, scheme) in items {
            checker_print!(
                self,
                "    {} {}.{} :: {}",
                "let".bold(),
                class_name,
                item_name,
                self.get_scheme_string(&scheme)
            );
        }
        checker_print!(self, "{}", "end".bold());

        ir::Stmt::Nothing
    }
}

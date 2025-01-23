use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Note, Report},
    Checker,
};
use either::Either;

impl<'src, 'e> Checker<'src, 'e> {
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

    pub fn check_class(&mut self, e: &ast::Class) -> ir::Stmt {
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

        use ast::ClassItem as K;
        use ast::Pattern as P;
        for (kind, lhs, rhs) in &e.items {
            match self.check_pattern_or_type_signature(lhs) {
                Either::Left(pattern) => {
                    let P::Binding(item_name_span) = pattern else {
                        panic!("invalid constant item syntax in class");
                    };

                    if !matches!(kind, K::Constant | K::Unknown) {
                        panic!("invalid type syntax for constant item in class (must use ':', not '=>')")
                    }

                    let item_name = item_name_span.lexeme(self.source);
                    let item_type = self.check_type(rhs);

                    let scheme = self.generalize_type(item_type);
                    println!(
                        "{class_name}.{item_name} :: {}",
                        self.get_scheme_string(&scheme)
                    );
                }
                Either::Right((signature, name_span)) => {
                    let Some(item_name_span) = name_span else {
                        panic!("invalid function item syntax in class (no name)")
                    };

                    let item_name = item_name_span.lexeme(self.source);
                    let (item_type, sig_ret_type) = self.declare_type_signature(&signature);
                    let ret_type = self.check_type(rhs);
                    self.unify(ret_type, sig_ret_type, &[]);

                    let scheme = self.generalize_type(item_type);
                    println!(
                        "{class_name}.{item_name} :: {}",
                        self.get_scheme_string(&scheme)
                    );
                }
            }
        }

        self.close_scope();

        ir::Stmt::Nothing
    }
}

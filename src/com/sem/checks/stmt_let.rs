use colored::Colorize;
use either::Either;

use crate::com::{
    Checker, ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    sem::checker::checker_print,
};

impl Checker<'_, '_> {
    fn check_pattern_or_signature(
        &mut self,
        e: &ast::Expr,
    ) -> Either<ast::Pattern, ast::Signature> {
        use ast::Expr as E;
        match e {
            E::Call(..) => Either::Right(self.check_signature(e, true)),
            _ => Either::Left(self.check_pattern(e)),
        }
    }

    pub fn check_let(&mut self, e: &ast::Let, public: bool) -> ir::Stmt {
        let (stmt, bindings) = self.check_let_bindings(e, public);

        for binding in bindings {
            let info = self.entities.get_variable_info(binding);
            let name = info.name.clone();
            let scheme = info.scheme.clone();
            checker_print!(
                self,
                "{} {name} :: {}",
                "let".bold(),
                self.get_scheme_string(&scheme)
            );
        }

        stmt
    }

    pub fn check_let_bindings(
        &mut self,
        e: &ast::Let,
        public: bool,
    ) -> (ir::Stmt, Vec<ir::VariableID>) {
        let binding_span = Span::combine(e.let_kw, e.pattern.span());
        let lhs = self.check_pattern_or_signature(&e.pattern);
        match lhs {
            Either::Left(pattern) => {
                if !pattern.is_irrefutable() {
                    self.reports.push(
                        Report::error(Header::RefutablePattern())
                            .with_primary_label(Label::Empty, pattern.span().wrap(self.file))
                            .with_secondary_label(
                                Label::LetBindingPattern,
                                binding_span.wrap(self.file),
                            ),
                    );
                }

                let (value, ty) = self.check_expression(&e.value);
                let (pattern, pattern_type) = self.declare_pattern(&pattern, public);

                self.unify(ty, pattern_type, &[]);
                let relevant_constraints = self.solve_constraints();

                let bindings = pattern.get_binding_ids();
                for var_id in bindings.iter().copied() {
                    let ty = self
                        .entities
                        .get_variable_info(var_id)
                        .scheme
                        .uninstantiated;

                    let mut scheme = self.generalize_type(ty);
                    for constraint in relevant_constraints.clone() {
                        self.add_class_constraint(&mut scheme, constraint);
                    }

                    self.entities.get_variable_info_mut(var_id).scheme = scheme.clone();
                    self.set_entity_public(var_id.wrap(), public);
                }

                (
                    ir::Stmt::Let {
                        lhs: pattern,
                        rhs: value,
                    },
                    bindings,
                )
            }
            Either::Right(signature) => {
                for arg_pattern in signature.arg_patterns() {
                    if !arg_pattern.is_irrefutable() {
                        self.reports.push(
                            Report::error(Header::RefutablePattern())
                                .with_primary_label(
                                    Label::Empty,
                                    arg_pattern.span().wrap(self.file),
                                )
                                .with_secondary_label(
                                    Label::FunctionArgPattern,
                                    binding_span.wrap(self.file),
                                ),
                        );
                    }
                }

                self.open_scope(true);

                let name = self.signature_name(&signature);
                if let Some((name, _)) = name {
                    self.set_scope_name(name.to_string());
                }

                let (sig, sig_type, ret_type, rec_id) = self.declare_signature(&signature);
                self.set_type_span(sig_type, e.pattern.span());
                self.set_type_span(ret_type, e.value.span());

                let (val, val_type) = self.check_expression(&e.value);
                self.unify(val_type, ret_type, &[]);
                let relevant_contraints = self.solve_constraints();

                let full_function_name = self.build_scope_name();
                self.close_scope();

                let Some((name, name_span)) = name else {
                    if !matches!(signature, ast::Signature::Missing) {
                        self.reports.push(
                            Report::error(Header::InvalidSignature()).with_primary_label(
                                Label::NamelessSignature,
                                e.pattern.span().wrap(self.file),
                            ),
                        );
                    }
                    return (ir::Stmt::Missing, vec![]);
                };

                let mut scheme = self.generalize_type(sig_type);
                for constraint in relevant_contraints {
                    self.add_class_constraint(&mut scheme, constraint);
                }

                let var_id = self.create_variable_poly(name, scheme, name_span);
                self.set_entity_public(var_id.wrap(), public);
                let pattern = ir::Pattern::Binding(var_id);
                let lambda = ir::Expr::Fun {
                    name: full_function_name,
                    recursive_binding: rec_id,
                    signature: Box::new(sig),
                    expr: Box::new(val),
                };

                (
                    ir::Stmt::Let {
                        lhs: pattern,
                        rhs: lambda,
                    },
                    vec![var_id],
                )
            }
        }
    }
}

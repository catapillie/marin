use crate::com::{
    Checker, ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
};

impl<'src> Checker<'src, '_> {
    pub fn check_signature(&mut self, mut e: &ast::Expr, require_name: bool) -> ast::Signature {
        use ast::Signature as S;
        let mut signature = S::Empty;
        loop {
            use ast::Expr as E;
            match e {
                E::Var(lex) if !require_name || !matches!(signature, S::Empty) => {
                    return S::Name(lex.span, Box::new(signature));
                }
                E::Tuple(tuple) => {
                    return S::Args(
                        tuple
                            .items
                            .iter()
                            .map(|arg| self.check_pattern(arg))
                            .collect(),
                        Box::new(signature),
                    );
                }
                E::Call(call) => {
                    let patterns = call
                        .args
                        .iter()
                        .map(|arg| self.check_pattern(arg))
                        .collect();
                    e = &call.callee;
                    signature = S::Args(patterns, Box::new(signature));
                }
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidSignature())
                            .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                    );
                    return S::Missing;
                }
            }
        }
    }

    pub fn check_type_signature(
        &mut self,
        mut e: &ast::Expr,
    ) -> (ast::TypeSignature, Option<Span>) {
        use ast::TypeSignature as S;
        let mut signature = S::Empty;
        loop {
            use ast::Expr as E;
            match e {
                E::Var(lex) => return (signature, Some(lex.span)),
                E::Tuple(tuple) => {
                    return (
                        S::Args(tuple.items.iter().cloned().collect(), Box::new(signature)),
                        None,
                    );
                }
                E::Call(call) => {
                    let patterns = call.args.iter().cloned().collect();
                    e = &call.callee;
                    signature = S::Args(patterns, Box::new(signature));
                }
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidTypeSignature())
                            .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                    );
                    return (S::Missing, None);
                }
            }
        }
    }

    pub fn signature_name(&self, s: &ast::Signature) -> Option<(&'src str, Span)> {
        use ast::Signature as S;
        match s {
            S::Name(span, _) => Some((span.lexeme(self.source), *span)),
            _ => None,
        }
    }

    // (signature, sig_type, ret_type, recursive_binding)
    pub fn declare_signature(
        &mut self,
        s: &ast::Signature,
    ) -> (
        ir::Signature,
        ir::TypeID,
        ir::TypeID,
        Option<ir::VariableID>,
    ) {
        use ast::Signature as S;
        use ir::Signature as I;
        match s {
            S::Missing => (
                I::Missing,
                self.create_fresh_type(None),
                self.create_fresh_type(None),
                None,
            ),
            S::Name(span, next) => {
                let (sig, sig_type, ret_type, _) = self.declare_signature(next);
                let name = span.lexeme(self.source);
                let id = self.create_variable_mono(name, sig_type, *span);
                (sig, sig_type, ret_type, Some(id))
            }
            S::Args(patterns, next) => {
                let (sig, sig_type, ret_type, _) = self.declare_signature(next);
                let (arg_patterns, arg_types): (Vec<_>, Vec<_>) = patterns
                    .iter()
                    .map(|arg| self.declare_pattern(arg, false))
                    .unzip();
                (
                    I::Args {
                        args: arg_patterns.into(),
                        next: Box::new(sig),
                    },
                    self.create_type(ir::Type::Lambda(arg_types.into(), sig_type), None),
                    ret_type,
                    None,
                )
            }
            S::Empty => {
                let ret_type = self.create_fresh_type(None);
                (I::Done, ret_type, ret_type, None)
            }
        }
    }

    // (sig_type, ret_type)
    pub fn declare_type_signature(&mut self, s: &ast::TypeSignature) -> (ir::TypeID, ir::TypeID) {
        use ast::TypeSignature as S;
        match s {
            S::Missing => (self.create_fresh_type(None), self.create_fresh_type(None)),
            S::Args(exprs, next) => {
                let (sig_type, ret_type) = self.declare_type_signature(next);
                let arg_types = exprs.iter().map(|arg| self.check_type(arg)).collect();
                (
                    self.create_type(ir::Type::Lambda(arg_types, sig_type), None),
                    ret_type,
                )
            }
            S::Empty => {
                let ret_type = self.create_fresh_type(None);
                (ret_type, ret_type)
            }
        }
    }

    // extract a signature with a curry depth of at most 1
    pub fn extract_simple_signature(e: &ast::Expr) -> Option<(Span, Option<&[ast::Expr]>)> {
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

    // extract a signature with a curry depth of exactly 1
    pub fn extract_simple_signature_with_args(e: &ast::Expr) -> Option<(Span, &[ast::Expr])> {
        use ast::Expr as E;
        match e {
            E::Call(call) => match &*call.callee {
                E::Var(e) => Some((e.span, &*call.args)),
                _ => None,
            },
            _ => None,
        }
    }
}

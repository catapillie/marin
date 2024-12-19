use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_call_type(&mut self, t: &ast::Call) -> ir::TypeID {
        // this is currently quite simple, but will need to be generalized
        // (only works for union types)
        let args: Box<_> = t.args.iter().map(|arg| self.check_type(arg)).collect();

        use ast::Expr as E;
        let E::Var(lex) = &*t.callee else {
            self.reports.push(
                Report::error(Header::InvalidType())
                    .with_primary_label(Label::Empty, t.span().wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span()));
        };

        let name_span = lex.span;
        let name = name_span.lexeme(self.source);

        let Some(id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownType(name.to_string()))
                    .with_primary_label(Label::Empty, name_span.wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span()));
        };
        let id = *id;

        let ir::Entity::Type(info) = &self.entities[id.0] else {
            self.reports.push(
                Report::error(Header::NotType(name.to_string()))
                    .with_primary_label(Label::Empty, name_span.wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span()));
        };

        let ir::TypeInfo::Union(info) = info else {
            self.reports.push(
                Report::error(Header::InvalidType())
                    .with_primary_label(Label::Empty, t.span().wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span()));
        };

        let Some(type_args) = &info.type_args else {
            self.reports.push(
                Report::error(Header::InvalidType())
                    .with_primary_label(Label::Empty, t.span().wrap(self.file)),
            );
            return self.create_fresh_type(Some(t.span()));
        };

        let arity = type_args.len();
        let union_ty = self.instantiate_scheme(info.scheme.clone());
        let union_ty = self.clone_type_repr(union_ty);
        self.set_type_span(union_ty, t.span());

        if arity == args.len() {
            let ty = self.create_type(ir::Type::Union(id, Some(args)), Some(t.span()));
            self.unify(ty, union_ty, &[]);
        } else {
            self.reports.push(
                Report::error(Header::InvalidType())
                    .with_primary_label(Label::Empty, t.span().wrap(self.file)),
            );
        }

        union_ty
    }
}

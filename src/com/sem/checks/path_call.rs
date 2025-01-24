use super::path::PathQuery as Q;
use crate::com::{
    ast, ir,
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_call_path(&mut self, e: &ast::Call) -> Q {
        let q = self.check_path(&e.callee);
        match q {
            Q::Missing => Q::Missing,

            // should be handled by other functions
            Q::Expr(_) => todo!("call on expr"),
            Q::Variant(_, _) => todo!("call on variant"),

            Q::Type(_) => todo!("call on type"),
            Q::Record(id) => self.check_record_call_path(id, &e.args, e.span()),
            Q::Union(id) => self.check_union_call_path(id, &e.args, e.span()),
            Q::Class(_) => todo!("call on class"),
        }
    }

    fn check_union_call_path(&mut self, id: ir::EntityID, args: &[ast::Expr], span: Span) -> Q {
        let args = args
            .iter()
            .map(|ty| self.check_type(ty))
            .collect::<Box<_>>();

        let info = self.get_union_info(id);
        let Some(type_args) = &info.type_args else {
            self.reports.push(
                Report::error(Header::UnionArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::UnionTypeNoArgs(info.name.to_string()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::UnionDefinition(info.name.to_string()), info.loc),
            );
            return Q::Missing;
        };

        let arity = type_args.len();
        let union_ty = self.instantiate_scheme(info.scheme.clone());
        let union_ty = self.clone_type_repr(union_ty);
        self.set_type_span(union_ty, span);

        let info = self.get_union_info(id);
        if arity == args.len() {
            let ty = self.create_type(ir::Type::Union(id, Some(args)), Some(span));
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
            return Q::Missing;
        }

        Q::Type(union_ty)
    }

    fn check_record_call_path(&mut self, id: ir::EntityID, args: &[ast::Expr], span: Span) -> Q {
        let args = args
            .iter()
            .map(|ty| self.check_type(ty))
            .collect::<Box<_>>();

        let info = self.get_record_info(id);
        let Some(type_args) = &info.type_args else {
            self.reports.push(
                Report::error(Header::RecordArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::RecordTypeNoArgs(info.name.to_string()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(info.name.to_string()), info.loc),
            );
            return Q::Missing;
        };

        let arity = type_args.len();
        let record_ty = self.instantiate_scheme(info.scheme.clone());
        let record_ty = self.clone_type_repr(record_ty);
        self.set_type_span(record_ty, span);

        let info = self.get_record_info(id);
        if arity == args.len() {
            let ty = self.create_type(ir::Type::Record(id, Some(args)), Some(span));
            self.unify(ty, record_ty, &[]);
        } else {
            self.reports.push(
                Report::error(Header::RecordArgMismatch(info.name.to_string()))
                    .with_primary_label(
                        Label::RecordTypeArgCount(info.name.to_string(), arity),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::RecordDefinition(info.name.to_string()), info.loc),
            );
            return Q::Missing;
        }

        Q::Type(record_ty)
    }
}

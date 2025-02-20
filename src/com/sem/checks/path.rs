use crate::com::{
    ast,
    ir::{self, TypeProvenance},
    loc::Span,
    reporting::{Header, Label, Report},
    Checker,
};

#[allow(dead_code)]
pub enum PathQuery {
    Missing,
    Expr(ir::CheckedExpr),
    Type(ir::TypeID),
    Record(ir::EntityID),
    Union(ir::EntityID),
    Variant(ir::EntityID, usize),
    Class(ir::EntityID),
}

use PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_path(&mut self, e: &ast::Expr) -> Q {
        use ast::Expr as E;
        match e {
            E::Var(e) => self.check_var_path(e),
            E::Access(e) => self.check_access_path(e),
            E::Call(e) => self.check_call_path(e),
            _ => Q::Expr(self.check_expression(e)),
        }
    }

    pub fn check_path_into_expr(&mut self, q: Q, span: Span) -> ir::CheckedExpr {
        match q {
            Q::Missing => self.check_missing(),
            Q::Expr(e) => e,
            Q::Variant(id, tag) => self.check_variant_path_into_expr(id, tag, span),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::NotAnExpression, span.wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }

    fn check_variant_path_into_expr(
        &mut self,
        id: ir::EntityID,
        tag: usize,
        span: Span,
    ) -> ir::CheckedExpr {
        let info = self.get_union_info(id);

        let variant = &info.variants[tag];
        let provenance = TypeProvenance::VariantDefinition(
            variant.loc,
            variant.name.clone(),
            info.loc,
            info.name.clone(),
        );

        let expr = variant.expr.clone();
        let ty = self.instantiate_scheme(variant.scheme.clone(), None);
        let ty = self.clone_type_repr(ty);
        self.set_type_span(ty, span);
        self.add_type_provenance(ty, provenance);

        (expr, ty)
    }

    pub fn check_path_into_type(&mut self, q: Q, span: Span) -> ir::TypeID {
        match q {
            Q::Missing => self.create_fresh_type(Some(span)),
            Q::Type(ty) => ty,
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidType())
                        .with_primary_label(Label::NotAType, span.wrap(self.file)),
                );
                self.create_fresh_type(Some(span))
            }
        }
    }
}

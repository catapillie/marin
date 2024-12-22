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
    Union(ir::EntityID),
    Variant(ir::EntityID, usize),
}

use PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_path(&mut self, e: &ast::Expr) -> Q {
        use ast::Expr as E;
        match e {
            E::Var(e) => self.check_var_path(e),
            E::Access(e) => self.check_access_path(e),
            _ => Q::Expr(self.check_expression(e)),
        }
    }

    pub fn check_path_into_expr(&mut self, q: Q, span: Span) -> ir::CheckedExpr {
        match q {
            Q::Missing => self.check_missing(),
            Q::Expr(e) => e,
            Q::Variant(id, tag) => self.check_variant_path_into_expr(id, tag),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::NotAnExpression, span.wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }

    fn check_variant_path_into_expr(&mut self, id: ir::EntityID, tag: usize) -> ir::CheckedExpr {
        let ir::Entity::Type(ir::TypeInfo::Union(info)) = self.get_entity(id) else {
            unreachable!("id '{}' is not that of a union type", id.0)
        };

        let variant = &info.variants[tag];

        let provenance = TypeProvenance::VariantDefinition(
            variant.loc,
            variant.name.clone(),
            info.loc,
            info.name.clone(),
        );

        let expr = variant.expr.clone();
        let ty = self.instantiate_scheme(variant.scheme.clone());
        self.add_type_provenance(ty, provenance);

        (expr, ty)
    }
}

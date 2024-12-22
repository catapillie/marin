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
    fn check_identifier_accessor(&mut self, e: &ast::Expr) -> Option<(&'src str, Span)> {
        use ast::Expr as E;
        match e {
            E::Var(e) => Some((e.span.lexeme(self.source), e.span)),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidAccessor())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                None
            }
        }
    }

    pub fn check_path(&mut self, e: &ast::Expr) -> Q {
        use ast::Expr as E;
        match e {
            E::Var(e) => self.check_var_path(e),
            E::Access(e) => self.check_access_path(e),
            _ => Q::Expr(self.check_expression(e)),
        }
    }

    pub fn check_var_path(&mut self, e: &ast::Lexeme) -> Q {
        let name = e.span.lexeme(self.source);
        let Some(id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownBinding(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return Q::Missing;
        };

        use ir::Entity as Ent;
        use ir::TypeInfo as T;
        match self.get_entity(*id) {
            Ent::Dummy => unreachable!(),
            Ent::Variable(_) => Q::Expr(self.check_var(e)),
            Ent::Type(T::Type(id)) => Q::Type(*id),
            Ent::Type(T::Union(_)) => Q::Union(*id),
        }
    }

    pub fn check_access_path(&mut self, e: &ast::Access) -> Q {
        let q = self.check_path(&e.accessed);
        match q {
            Q::Missing => Q::Missing,
            Q::Expr(_) => todo!("access on expr"),
            Q::Type(_) => todo!("access on type"),
            Q::Union(id) => self.check_union_access_path(id, &e.accessor),
            Q::Variant(_, _) => todo!("access on variant"),
        }
    }

    pub fn check_union_access_path(&mut self, id: ir::EntityID, accessor: &ast::Expr) -> Q {
        let Some((name, name_span)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let ir::Entity::Type(ir::TypeInfo::Union(info)) = self.get_entity(id) else {
            unreachable!("id '{}' is not that of a union type", id.0)
        };

        let Some((tag, _)) = info
            .variants
            .iter()
            .enumerate()
            .find(|(_, var)| var.name == name)
        else {
            self.reports.push(
                Report::error(Header::UnknownVariant(name.to_string(), info.name.clone()))
                    .with_primary_label(Label::Empty, name_span.wrap(self.file))
                    .with_secondary_label(Label::UnionDefinition(info.name.clone()), info.loc),
            );
            return Q::Missing;
        };

        PathQuery::Variant(id, tag)
    }

    pub fn check_access(&mut self, e: &ast::Access) -> ir::CheckedExpr {
        let q = self.check_access_path(e);
        self.check_path_into_expr(q, e.span())
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

    pub fn check_variant_path_into_expr(
        &mut self,
        id: ir::EntityID,
        tag: usize,
    ) -> ir::CheckedExpr {
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

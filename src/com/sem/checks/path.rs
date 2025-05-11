use crate::com::{
    Checker, ast,
    ir::{self, TypeProvenance},
    loc::Span,
    reporting::{Header, Label, Report},
};

use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_path_or_type(&mut self, e: &ast::Expr) -> Q {
        use ast::Expr as E;
        match e {
            E::Var(e) => self.check_var_path(e),
            E::Access(e) => self.check_access_path(e),
            E::Call(e) => self.check_call_path(e),
            _ => Q::Type(self.check_type(e)),
        }
    }

    pub fn check_path_or_expr(&mut self, e: &ast::Expr) -> Q {
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
            Q::Var(id) => self.check_var_path_into_expr(id, span),
            Q::Expr(e) => e,
            Q::Variant(id, tag) => self.check_variant_path_into_expr(id, tag, span),
            Q::ClassItem(id, index) => self.check_class_item_into_expr(id, index, span),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::NotAnExpression, span.wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }

    fn check_var_path_into_expr(&mut self, id: ir::VariableID, span: Span) -> ir::CheckedExpr {
        let info = self.entities.get_variable_info(id);
        let name = info.name.to_string();
        let loc = info.loc;

        let is_concrete = info.scheme.constraints.is_empty();

        let info = self.entities.get_variable_info(id);
        let (constraint_id, instantiated) =
            self.instantiate_scheme(info.scheme.clone(), Some(span.wrap(self.file)));
        let ty = self.clone_type_repr(instantiated);
        self.set_type_span(ty, span);
        self.add_type_provenance(ty, TypeProvenance::VariableDefinition(loc, name));

        let expr = match is_concrete {
            false => ir::Expr::AbstractVar { id, constraint_id },
            true => ir::Expr::Var { id },
        };

        (expr, ty)
    }

    fn check_variant_path_into_expr(
        &mut self,
        id: ir::UnionID,
        tag: usize,
        span: Span,
    ) -> ir::CheckedExpr {
        let (info, variant) = self.entities.get_union_variant_info(id, tag);
        let provenance = TypeProvenance::VariantDefinition(
            variant.loc,
            variant.name.clone(),
            info.loc,
            info.name.clone(),
        );

        let expr = variant.expr.clone();
        let (_, ty) = self.instantiate_scheme(variant.scheme.clone(), None);
        let ty = self.clone_type_repr(ty);
        self.set_type_span(ty, span);
        self.add_type_provenance(ty, provenance);

        (expr, ty)
    }

    fn check_class_item_into_expr(
        &mut self,
        id: ir::ClassID,
        index: usize,
        span: Span,
    ) -> ir::CheckedExpr {
        let info = self.entities.get_class_info(id);

        let class_loc = info.loc;
        let class_name = info.name.clone();

        let item_info = self.entities.get_class_item_info(id, index);
        let item_loc = item_info.loc;
        let item_name = item_info.name.clone();

        let scheme = item_info.scheme.clone();

        let (constraint_id, item_ty) = self.instantiate_scheme(scheme, Some(span.wrap(self.file)));
        let item_ty = self.clone_type_repr(item_ty);
        self.set_type_span(item_ty, span);
        self.add_type_provenance(
            item_ty,
            ir::TypeProvenance::ClassItemDefinition(item_loc, item_name, class_loc, class_name),
        );

        (
            ir::Expr::ClassItem {
                item_id: index,
                constraint_id,
            },
            item_ty,
        )
    }

    pub fn check_path_into_type(&mut self, q: Q, span: Span) -> ir::TypeID {
        match q {
            Q::Missing => self.create_fresh_type(Some(span)),
            Q::Type(ty) => ty,
            Q::Union(union_id) => match self.create_union_type(union_id, None, span) {
                Some(ty) => ty,
                None => self.create_fresh_type(Some(span)),
            },
            Q::Record(record_ty) => match self.create_record_type(record_ty, None, span) {
                Some(ty) => ty,
                None => self.create_fresh_type(Some(span)),
            },
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

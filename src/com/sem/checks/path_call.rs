use crate::com::{Checker, ast, ir, loc::Span};
use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_call_path(&mut self, e: &ast::Call) -> Q {
        let q = self.check_path_or_expr(&e.callee);
        match q {
            Q::Missing => Q::Missing,
            Q::Expr(_) => todo!("call on expr"),
            Q::Var(_) => todo!("call on var"),
            Q::Variant(_, _) => todo!("call on variant"),
            Q::Type(_) => todo!("call on type"),
            Q::Record(id) => self.check_record_call_path(id, &e.args, e.span()),
            Q::Union(id) => self.check_union_call_path(id, &e.args, e.span()),
            Q::Class(_) => todo!("call on class"),
            Q::ClassItem(_, _) => todo!("call on class item"),
            Q::Import(_) => todo!("call on import"),
        }
    }

    fn check_union_call_path(
        &mut self,
        union_id: ir::UnionID,
        args: &[ast::Expr],
        span: Span,
    ) -> Q {
        let args = args
            .iter()
            .map(|ty| self.check_type(ty))
            .collect::<Box<_>>();
        match self.create_union_type(union_id, Some(args), span) {
            Some(union_ty) => Q::Type(union_ty),
            None => Q::Missing,
        }
    }

    fn check_record_call_path(
        &mut self,
        record_id: ir::RecordID,
        args: &[ast::Expr],
        span: Span,
    ) -> Q {
        let args = args
            .iter()
            .map(|ty| self.check_type(ty))
            .collect::<Box<_>>();
        match self.create_record_type(record_id, Some(args), span) {
            Some(record_ty) => Q::Type(record_ty),
            None => Q::Missing,
        }
    }
}

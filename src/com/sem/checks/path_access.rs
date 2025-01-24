use super::path::PathQuery as Q;
use crate::com::{ast, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_access_path(&mut self, e: &ast::Access) -> Q {
        let q = self.check_path(&e.accessed);
        match q {
            Q::Missing => Q::Missing,
            Q::Expr(_) => todo!("access on expr"),
            Q::Type(_) => todo!("access on type"),
            Q::Record(_) => todo!("access on record type"),
            Q::Union(id) => self.check_union_access_path(id, &e.accessor),
            Q::Variant(_, _) => todo!("access on variant"),
            Q::Class(_) => todo!("access on class"),
        }
    }
}

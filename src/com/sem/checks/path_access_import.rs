use crate::com::{ast, ir, Checker};

use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_import_access_path(&mut self, id: ir::EntityID, accessor: &ast::Expr) -> Q {
        let Some((name, _)) = self.check_identifier_accessor(accessor) else {
            return Q::Missing;
        };

        let file = self.get_import_info(id).file;
        todo!("access on import of file #{file} by '{name}'")
    }
}

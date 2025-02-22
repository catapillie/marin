use crate::com::{ast, ir, Checker};

use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_alias(&mut self, e: &ast::Alias) -> ir::Stmt {
        let q = self.check_path(&e.path);
        match q {
            Q::Missing => return ir::Stmt::Nothing,
            Q::Expr(_) => {
                panic!("alias cannot be used on expression")
            }
            _ => {}
        };

        ir::Stmt::Nothing
    }
}

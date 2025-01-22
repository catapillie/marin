use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_class(&mut self, e: &ast::Class) -> ir::Stmt {
        println!("class {:?} of {:?}", e.signature, e.associated);
        ir::Stmt::Nothing
    }
}

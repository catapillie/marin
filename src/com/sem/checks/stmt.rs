use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_statement(&mut self, e: &ast::Expr) -> ir::Stmt {
        use ast::Expr as E;
        match e {
            E::Let(e) => self.check_let(e),
            E::Alias(e) => self.check_alias(e),
            E::Import(e) => self.check_import(e),
            E::Record(e) => self.check_record(e),
            E::Union(e) => self.check_union(e),
            E::Class(e) => self.check_class(e),
            E::Have(e) => self.check_have(e),
            _ => {
                let (expr, ty) = self.check_expression(e);
                ir::Stmt::Expr(expr, ty)
            }
        }
    }

    pub fn check_file(&mut self, ast: &ast::File) -> ir::File {
        let stmts = ast.0.iter().map(|e| self.check_statement(e)).collect();

        let constraints = self.solve_constraints();
        if !constraints.is_empty() {
            panic!("unsolved constraints remain in the module");
        }

        ir::File { stmts }
    }
}

use crate::com::{ast, ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_file(
        &mut self,
        file_id: usize,
        file_source: &'src str,
        ast: &ast::File,
    ) -> ir::File {
        self.file = file_id;
        self.source = file_source;

        self.open_scope(true);

        let stmts = ast.0.iter().map(|e| self.check_statement(e)).collect();
        let constraints = self.solve_constraints();
        if !constraints.is_empty() {
            panic!("unsolved constraints remain in the file");
        }

        self.close_scope();

        ir::File { stmts }
    }
}

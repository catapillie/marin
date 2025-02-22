use colored::Colorize;

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

        let (exports, instances) = self.get_public_exports_and_instances();
        self.close_scope();

        if !exports.is_empty() || !instances.is_empty() {
            eprintln!("{}", "\nexport".bold());
            for id in &exports {
                eprintln!(" {}", self.get_entity_display(*id))
            }
            for id in &instances {
                eprintln!(" {}", self.get_entity_display(*id))
            }
            eprintln!("{}", "end".bold());
        }

        ir::File { stmts }
    }

    // (exports, instances)
    fn get_public_exports_and_instances(&self) -> (Vec<ir::EntityID>, Vec<ir::EntityID>) {
        let mut exports = Vec::new();
        for (_, id) in self.scope.iter() {
            if self.is_entity_public(*id) {
                exports.push(*id);
            }
        }

        let mut instances = Vec::new();
        for id in self.scope.infos_iter().flatten() {
            if self.is_entity_public(*id) {
                instances.push(*id);
            }
        }

        (exports, instances)
    }
}

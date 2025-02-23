use crate::com::{
    ast, ir,
    sem::checker::{checker_print, Export},
    Checker,
};
use colored::Colorize;
use std::collections::HashMap;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_module(
        &mut self,
        file_name: &str,
        file_id: usize,
        file_source: &'src str,
        ast: &ast::File,
        options: CheckModuleOptions,
    ) -> ir::Module {
        self.file = file_id;
        self.source = file_source;
        self.options = options;

        checker_print!(
            self,
            "\n{}",
            format!("=== checking '{}' ===", file_name)
                .on_bright_white()
                .black()
        );

        self.open_scope(true);

        let stmts = ast.0.iter().map(|e| self.check_statement(e)).collect();
        let constraints = self.solve_constraints();
        if !constraints.is_empty() {
            panic!("unsolved constraints remain in the file");
        }

        let (exports, instances) = self.get_public_exports_and_instances();
        self.close_scope();

        if !exports.is_empty() || !instances.is_empty() {
            checker_print!(self, "{}", "\nexport".bold());
            for id in exports.values() {
                checker_print!(self, "    {}", self.get_entity_display(*id))
            }
            for id in &instances {
                checker_print!(self, "    {}", self.get_entity_display(*id))
            }
            checker_print!(self, "{}", "end".bold());
        }

        self.exports[self.file] = Export {
            was_checked: true,
            exports,
            instances,
        };

        ir::Module { stmts }
    }

    // (exports, instances)
    fn get_public_exports_and_instances(
        &self,
    ) -> (HashMap<&'src str, ir::EntityID>, Vec<ir::EntityID>) {
        let mut exports = HashMap::new();
        for (name, id) in self.scope.iter() {
            if self.is_entity_public(*id) {
                exports.insert(*name, *id);
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

pub struct CheckModuleOptions {
    pub is_verbose: bool,
}

impl CheckModuleOptions {
    pub fn new() -> Self {
        Self { is_verbose: false }
    }

    pub fn set_verbose(mut self, is_verbose: bool) -> Self {
        self.is_verbose = is_verbose;
        self
    }
}

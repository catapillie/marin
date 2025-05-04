use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    sem::checker::{checker_print, Export},
    Checker,
};
use colored::Colorize;
use std::collections::HashMap;

impl<'src> Checker<'src, '_> {
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

        if self.options.import_prelude {
            self.import_std_prelude();
        }

        let mut stmts = Vec::with_capacity(ast.0.len());
        for e in &ast.0 {
            let stmt = self.check_statement(e);
            stmts.push(stmt);

            // constraints on a top-level statement cannot are unallowed
            // because such a statement cannot be compiled
            let constraints = self.solve_constraints();
            if !constraints.is_empty() {
                let constraint_strings = constraints
                    .iter()
                    .map(|constr| self.get_constraint_string(constr))
                    .collect::<Vec<_>>();

                let mut rep = Report::error(Header::TopLevelConstraint());
                for (constr, constr_str) in constraints.iter().zip(constraint_strings) {
                    rep = rep.with_secondary_label(Label::ConstraintOrigin(constr_str), constr.loc);
                }

                self.reports.push(
                    rep.with_primary_label(
                        Label::UnsatisfiedConstraints(constraints.len()),
                        e.span().wrap(self.file),
                    )
                    .with_note(Note::TopLevelUnknownTypes),
                );
            }
        }

        let (exports, instances) = self.get_public_exports_and_instances();
        self.close_scope();

        if !exports.is_empty() {
            checker_print!(self, "{}", "\nexport".bold());
            for id in exports.values() {
                checker_print!(self, "    {}", self.get_entity_display(*id))
            }
            checker_print!(self, "{}", "end".bold());
        }

        self.exports[self.file] = Export {
            was_checked: true,
            exports,
            instances,
        };

        ir::Module {
            stmts: stmts.into(),
        }
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
        for id in self.scope.infos_iter().flat_map(|info| &info.instances) {
            if self.is_entity_public(*id) {
                instances.push(*id);
            }
        }

        (exports, instances)
    }
}

pub struct CheckModuleOptions {
    pub is_verbose: bool,
    pub import_prelude: bool,
}

impl CheckModuleOptions {
    pub fn new() -> Self {
        Self {
            is_verbose: false,
            import_prelude: false,
        }
    }

    pub fn set_verbose(mut self, is_verbose: bool) -> Self {
        self.is_verbose = is_verbose;
        self
    }

    pub fn set_import_prelude(mut self, import_prelude: bool) -> Self {
        self.import_prelude = import_prelude;
        self
    }
}

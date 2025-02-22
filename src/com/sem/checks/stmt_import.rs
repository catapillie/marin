use crate::com::{ast, ir, Checker};
use colored::Colorize;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_import(&mut self, e: &ast::Import) -> ir::Stmt {
        use ast::Expr as E;
        for query in &e.queries {
            let file_name_span = match &*query.query {
                E::Var(lex) => lex.span,
                E::Access(e) => match &*e.accessor {
                    E::Var(lex) => lex.span,
                    _ => continue,
                },
                _ => continue,
            };

            let file_name = file_name_span.lexeme(self.source);
            let import_name = match query.alias {
                Some(span) => span.lexeme(self.source),
                None => file_name,
            };

            let Some(dep_file) = self
                .deps
                .edges(self.file)
                .find_map(|(_, file, uids)| match uids.contains(&query.uid) {
                    true => Some(file),
                    false => None,
                })
            else {
                continue;
            };

            eprintln!(
                "{} {file_name} {} '{import_name}'",
                "import".bold(),
                "as".bold()
            );

            let import_id = self.create_entity(ir::Entity::Import(ir::ImportInfo {
                name: import_name.to_string(),
                loc: e.span().wrap(self.file),
                file: dep_file,
            }));
            self.scope.insert(import_name, import_id);

            // import all instances of classes in our scope
            // make sure to copy them and reset them to private so they aren't exported from this file
            let dep_instances = self.exports[dep_file].instances.clone();
            for id in dep_instances {
                let instance_info = self.get_instance_info(id).clone();
                let instance_id = self.create_entity(ir::Entity::Instance(instance_info));
                self.scope.infos_mut().push(instance_id);
            }
        }

        ir::Stmt::Nothing
    }
}

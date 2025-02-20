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

            println!(
                "{} {file_name} {} '{import_name}'",
                "import".bold(),
                "as".bold()
            );

            let import_id =
                self.create_entity(ir::Entity::Import(ir::ImportInfo { file: dep_file }));
            self.scope.insert(import_name, import_id);
        }

        ir::Stmt::Nothing
    }
}

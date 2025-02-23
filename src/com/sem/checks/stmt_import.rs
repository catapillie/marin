use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};
use colored::Colorize;

use ast::Expr as E;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_import(&mut self, e: &ast::Import) -> ir::Stmt {
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

    pub fn check_import_from(&mut self, e: &ast::ImportFrom, public: bool) -> ir::Stmt {
        let file_name_span = match &*e.path_query {
            E::Var(lex) => lex.span,
            E::Access(e) => match &*e.accessor {
                E::Var(lex) => lex.span,
                _ => return ir::Stmt::Nothing,
            },
            _ => return ir::Stmt::Nothing,
        };

        let file_name = file_name_span.lexeme(self.source);

        let Some(dep_file) = self.deps.edges(self.file).find_map(|(_, file, uids)| {
            match uids.contains(&e.path_query_uid) {
                true => Some(file),
                false => None,
            }
        }) else {
            return ir::Stmt::Nothing;
        };

        eprintln!("{}", "import".bold());
        for query in &e.queries {
            let E::Var(item_name_span) = &*query.query else {
                self.reports.push(
                    Report::error(Header::InvalidItemQuery())
                        .with_primary_label(Label::Empty, query.query.span().wrap(self.file))
                        .with_note(Note::ItemQuerySyntax),
                );
                continue;
            };

            let item_name = item_name_span.span.lexeme(self.source);
            let import_name = match query.alias {
                Some(span) => span.lexeme(self.source),
                None => item_name,
            };

            let items = &self.exports[dep_file];
            if !items.was_checked {
                continue;
            }

            let path = match items.exports.get(item_name) {
                Some(id) => self.check_entity_into_path(*id),
                None => {
                    self.reports.push(
                        Report::error(Header::UnknownExport(
                            item_name.to_string(),
                            file_name.to_string(),
                        ))
                        .with_primary_label(Label::Empty, query.query.span().wrap(self.file)),
                    );
                    continue;
                }
            };

            let alias_id = self.create_entity(ir::Entity::Alias(ir::AliasInfo {
                name: import_name.to_string(),
                path,
            }));
            self.scope.insert(import_name, alias_id);
            self.set_entity_public(alias_id, public);

            eprintln!("    {item_name} {} '{import_name}'", "as".bold());
        }
        eprintln!("{} {file_name}", "from".bold());

        ir::Stmt::Nothing
    }
}

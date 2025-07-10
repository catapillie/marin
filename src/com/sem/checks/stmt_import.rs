use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Note, Report},
    sem::checker::checker_print,
};
use colored::Colorize;

use ast::Expr as E;

impl Checker<'_, '_> {
    pub fn check_import(&mut self, e: &ast::Import, public: bool) -> ir::Stmt {
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
                .graph
                .edges(self.file)
                .find_map(|(_, file, uids)| match uids.contains(&query.uid) {
                    true => Some(file),
                    false => None,
                })
            else {
                continue;
            };

            checker_print!(
                self,
                "{} {file_name} {} '{import_name}'",
                "import".bold(),
                "as".bold()
            );

            let import_id = self.entities.create_import(ir::ImportInfo {
                name: import_name.to_string(),
                loc: e.span().wrap(self.file),
                file: dep_file,
            });
            self.scope.insert(import_name, import_id.wrap());
            self.set_entity_public(import_id.wrap(), public);

            // import all instances of classes in our scope
            self.import_all_instances(dep_file);
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

        let Some(dep_file) =
            self.deps
                .graph
                .edges(self.file)
                .find_map(|(_, file, uids)| match uids.contains(&e.path_query_uid) {
                    true => Some(file),
                    false => None,
                })
        else {
            return ir::Stmt::Nothing;
        };

        // import instances
        self.import_all_instances(dep_file);

        checker_print!(self, "{}", "import".bold());
        for query in &e.queries {
            // // potential syntax to import everything from a file
            // if (???) {
            //     self.import_all_items(dep_file, public);
            //     checker_print!(self, "    ..");
            //     continue;
            // };

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

            let alias_id = self.entities.create_alias(ir::AliasInfo {
                name: import_name.to_string(),
                path,
            });
            self.scope.insert(import_name, alias_id.wrap());
            self.set_entity_public(alias_id.wrap(), public);

            checker_print!(self, "    {item_name} {} '{import_name}'", "as".bold());
        }
        checker_print!(self, "{} {file_name}", "from".bold());

        ir::Stmt::Nothing
    }

    fn import_all_items(&mut self, dep_file: usize, public: bool) {
        let dep_exports = self.exports[dep_file]
            .exports
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect::<Vec<_>>();
        for (name, id) in dep_exports {
            let path = self.check_entity_into_path(id);
            let alias_id = self.entities.create_alias(ir::AliasInfo {
                name: name.to_string(),
                path,
            });
            self.scope.insert(name, alias_id.wrap());
            self.set_entity_public(alias_id.wrap(), public);
        }
    }

    fn import_all_instances(&mut self, dep_file: usize) {
        let dep_instances = self.exports[dep_file].instances.clone();
        for id in dep_instances {
            self.scope.infos_mut().instances.insert(id);
        }
    }

    pub fn import_std_prelude(&mut self) {
        let Some(id) = self.deps.info.prelude_file else {
            return;
        };

        self.import_all_items(id, false);
        self.import_all_instances(id);
    }
}

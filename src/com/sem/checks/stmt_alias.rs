use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Report},
    sem::checker::checker_print,
};

use colored::Colorize;
use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_alias(&mut self, e: &ast::Alias, public: bool) -> ir::Stmt {
        let path = self.check_path_or_type(&e.path);

        // a bit hacky but prevents instances from appearing if the aliased item is an expression
        self.solve_constraints();

        match path {
            Q::Missing => return ir::Stmt::Nothing,
            Q::Expr(_) => {
                self.reports.push(
                    Report::error(Header::ExpressionAlias())
                        .with_primary_label(Label::Empty, e.path.span().wrap(self.file))
                        .with_secondary_label(
                            Label::CannotAliasExpression,
                            e.span().wrap(self.file),
                        ),
                );
                return ir::Stmt::Nothing;
            }
            _ => {}
        };

        let entity_desc = match &path {
            Q::Missing => unreachable!(),
            Q::Expr(_) => unreachable!(),
            Q::Var(id) => format!(
                "({}) {}",
                "variable".bold(),
                self.entities.get_variable_info(*id).name
            ),
            Q::Type(id) => format!("({}) {}", "type".bold(), self.get_type_string(*id)),
            Q::Record(id) => format!(
                "({}) {}",
                "record".bold(),
                self.entities.get_record_info(*id).name
            ),
            Q::Union(id) => format!(
                "({}) {}",
                "union".bold(),
                self.entities.get_union_info(*id).name
            ),
            Q::Variant(id, tag) => {
                let (info, variant_info) = self.entities.get_union_variant_info(*id, *tag);
                format!("({}) {}.{}", "variant".bold(), info.name, variant_info.name,)
            }
            Q::Class(id) => format!(
                "({}) {}",
                "class".bold(),
                self.entities.get_class_info(*id).name
            ),
            Q::ClassItem(id, index) => {
                let class_info = self.entities.get_class_info(*id);
                let item_info = self.entities.get_class_item_info(*id, *index);
                format!(
                    "({}) {}.{}",
                    "class item".bold(),
                    class_info.name,
                    item_info.name
                )
            }
            Q::Import(id) => format!(
                "({}) {}",
                "import".bold(),
                self.entities.get_import_info(*id).name
            ),
        };

        let alias_name = e.name.lexeme(self.source);
        let alias_id = self.entities.create_alias(ir::AliasInfo {
            name: alias_name.to_string(),
            path,
        });
        self.scope.insert(alias_name, alias_id.wrap());
        self.set_entity_public(alias_id.wrap(), public);

        checker_print!(
            self,
            "{}{entity_desc} {} {alias_name}",
            "alias".bold(),
            "as".bold()
        );

        ir::Stmt::Nothing
    }
}

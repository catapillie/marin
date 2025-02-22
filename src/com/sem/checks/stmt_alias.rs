use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_alias(&mut self, e: &ast::Alias) -> ir::Stmt {
        let path = self.check_path(&e.path);
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

        let alias_name = e.name.lexeme(self.source);
        let alias_id = self.create_entity(ir::Entity::Alias(ir::AliasInfo { path }));
        self.scope.insert(alias_name, alias_id);

        ir::Stmt::Nothing
    }
}

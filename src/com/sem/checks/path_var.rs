use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::PathQuery as Q;

impl Checker<'_, '_> {
    pub fn check_entity_into_path(&self, id: ir::AnyID) -> Q {
        use ir::AnyID as ID;
        match id {
            ID::Variable(id) => Q::Var(id),
            ID::UserType(id) => Q::Type(self.entities.get_user_type_info(id).id),
            ID::Record(id) => Q::Record(id),
            ID::Union(id) => Q::Union(id),
            ID::Class(id) => Q::Class(id),
            ID::Instance(_) => unreachable!(),
            ID::Import(id) => Q::Import(id),
            ID::Alias(id) => self.entities.get_alias_info(id).path.clone(),
        }
    }

    pub fn try_check_var_path(&self, e: &ast::Lexeme) -> Option<Q> {
        let name = e.span.lexeme(self.source);
        let id = self.scope.search(name)?;
        Some(self.check_entity_into_path(*id))
    }

    pub fn check_var_path(&mut self, e: &ast::Lexeme) -> Q {
        match self.try_check_var_path(e) {
            Some(q) => q,
            None => {
                let name = e.span.lexeme(self.source);
                self.reports.push(
                    Report::error(Header::UnknownBinding(name.to_string()))
                        .with_primary_label(Label::Empty, e.span.wrap(self.file)),
                );
                Q::Missing
            }
        }
    }
}

use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::Entity as Ent;
use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_entity_into_path(&self, id: ir::EntityID) -> Q {
        match self.get_entity(id) {
            Ent::Dummy => unreachable!(),
            Ent::Variable(_) => Q::Var(id),
            Ent::Type(id) => Q::Type(*id),
            Ent::Record(_) => Q::Record(id),
            Ent::Union(_) => Q::Union(id),
            Ent::Class(_) => Q::Class(id),
            Ent::Instance(_) => unreachable!(),
            Ent::Import(_) => Q::Import(id),
            Ent::Alias(info) => info.path.clone(),
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

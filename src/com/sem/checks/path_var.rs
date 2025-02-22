use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

use ir::PathQuery as Q;

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_var_path(&mut self, e: &ast::Lexeme) -> Q {
        let name = e.span.lexeme(self.source);
        let Some(id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownBinding(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return Q::Missing;
        };

        use ir::Entity as Ent;
        match self.get_entity(*id) {
            Ent::Dummy => unreachable!(),
            Ent::Variable(_) => Q::Var(*id),
            Ent::Type(id) => Q::Type(*id),
            Ent::Record(_) => Q::Record(*id),
            Ent::Union(_) => Q::Union(*id),
            Ent::Class(_) => Q::Class(*id),
            Ent::Instance(_) => unreachable!(),
            Ent::Import(_) => Q::Import(*id),
            Ent::Alias(info) => info.path.clone(),
        }
    }
}

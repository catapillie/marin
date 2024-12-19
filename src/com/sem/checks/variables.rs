use crate::com::{ir, loc::Span, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn create_variable_poly(
        &mut self,
        name: &'src str,
        scheme: ir::Scheme,
        span: Span,
    ) -> ir::EntityID {
        let id = self.create_entity(ir::Entity::Variable(ir::Variable {
            scheme,
            loc: span.wrap(self.file),
        }));
        self.scope.insert(name, id);
        id
    }

    pub fn create_variable_mono(
        &mut self,
        name: &'src str,
        ty: ir::TypeID,
        span: Span,
    ) -> ir::EntityID {
        self.create_variable_poly(name, ir::Scheme::mono(ty), span)
    }

    pub fn get_variable(&self, id: ir::EntityID) -> &ir::Variable {
        match &self.entities[id.0] {
            ir::Entity::Variable(v) => v,
            _ => panic!("id '{}' is not that of an entity", id.0),
        }
    }

    pub fn get_variable_mut(&mut self, id: ir::EntityID) -> &mut ir::Variable {
        match &mut self.entities[id.0] {
            ir::Entity::Variable(v) => v,
            _ => panic!("id '{}' is not that of an entity", id.0),
        }
    }
}

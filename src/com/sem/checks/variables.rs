use crate::com::{Checker, ir, loc::Span};

impl<'src> Checker<'src, '_> {
    pub fn create_variable_poly(
        &mut self,
        name: &'src str,
        scheme: ir::Scheme,
        span: Span,
    ) -> ir::VariableID {
        let id = self.entities.create_variable(ir::VariableInfo {
            name: name.to_string(),
            scheme,
            loc: span.wrap(self.file),
        });
        self.scope.insert(name, id.wrap());
        id
    }

    pub fn create_variable_mono(
        &mut self,
        name: &'src str,
        ty: ir::TypeID,
        span: Span,
    ) -> ir::VariableID {
        self.create_variable_poly(name, ir::Scheme::mono(ty), span)
    }
}

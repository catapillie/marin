use crate::com::{ast, ir, reporting::Label, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_have(&mut self, e: &ast::Have) -> ir::Stmt {
        let class_name = e.name.lexeme(self.source);
        let within_label = Label::WithinClassInstantiation(class_name.to_string());

        use super::path::PathQuery as Q;
        let lexeme = ast::Lexeme { span: e.name };
        let class_id = match self.check_var_path(&lexeme) {
            Q::Class(id) => id,
            _ => return ir::Stmt::Nothing,
        };

        let info = self.get_class_info(class_id);

        self.open_scope(false);

        use ast::Expr as E;
        for item in &e.items {
            let E::Let(item) = item else {
                todo!("invalid instantiation item");
                continue;
            };

            let (_, bindings) = self.check_let_bindings(item);
        }

        self.close_scope();

        ir::Stmt::Nothing
    }
}

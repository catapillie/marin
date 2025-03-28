use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl Checker<'_, '_> {
    pub fn check_expression_list<'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a ast::Expr>,
    ) -> (Vec<ir::Expr>, Vec<ir::TypeID>) {
        iter.into_iter().map(|e| self.check_expression(e)).unzip()
    }

    pub fn check_expression(&mut self, e: &ast::Expr) -> ir::CheckedExpr {
        use ast::Expr as E;
        match e {
            E::Missing(_) => self.check_missing(),
            E::Int(e) => self.check_int(e),
            E::Float(e) => self.check_float(e),
            E::String(e) => self.check_string(e),
            E::True(e) => self.check_bool(e, true),
            E::False(e) => self.check_bool(e, false),
            E::Var(e) => self.check_var(e),
            E::Tuple(e) => self.check_tuple(e),
            E::Array(e) => self.check_array(e),
            E::Block(e) => self.check_block(e),
            E::Conditional(e) => self.check_conditional(e),
            E::Break(e) => self.check_break(e),
            E::Skip(e) => self.check_skip(e),
            E::Call(e) => self.check_call(e),
            E::Access(e) => self.check_access(e),
            E::Fun(e) => self.check_fun(e),
            E::RecordValue(e) => self.check_record_value(e),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }

    pub fn check_missing(&mut self) -> ir::CheckedExpr {
        (ir::Expr::Missing, self.create_fresh_type(None))
    }
}

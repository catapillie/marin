use crate::com::{
    ast, ir,
    reporting::{Header, Label, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_pattern(&mut self, e: &ast::Expr) -> ast::Pattern {
        use ast::Expr as E;
        use ast::Pattern as P;
        match e {
            E::Missing(e) => P::Missing(e.span),
            E::Var(e) => P::Binding(e.span),
            E::Int(e) => P::Int(e.span),
            E::Float(e) => P::Float(e.span),
            E::String(e) => P::String(e.span),
            E::True(e) => P::True(e.span),
            E::False(e) => P::False(e.span),
            E::Tuple(e) if e.items.len() == 1 => self.check_pattern(&e.items[0]),
            E::Tuple(e) => P::Tuple(
                e.left_paren,
                e.right_paren,
                e.items
                    .iter()
                    .map(|item| self.check_pattern(item))
                    .collect(),
            ),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidPattern())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                P::Missing(e.span())
            }
        }
    }

    pub fn declare_pattern(&mut self, p: &ast::Pattern) -> (ir::Pattern, ir::TypeID) {
        use ast::Pattern as P;
        use ir::Pattern as I;
        let span = p.span();
        match p {
            P::Missing(_) => (I::Missing, self.create_fresh_type(Some(span))),
            P::Binding(_) => {
                let name = span.lexeme(self.source);
                let ty = self.create_fresh_type(Some(span));
                let id = self.create_variable_mono(name, ty, span);
                (I::Binding(id), ty)
            }
            P::Int(_) => (
                self.read_source_int(span).map(I::Int).unwrap_or(I::Missing),
                self.create_type(ir::Type::Int, Some(span)),
            ),
            P::Float(_) => (
                self.read_source_float(span)
                    .map(I::Float)
                    .unwrap_or(I::Missing),
                self.create_type(ir::Type::Float, Some(span)),
            ),
            P::String(_) => (
                I::String(self.read_source_string(span).to_string()),
                self.create_type(ir::Type::Float, Some(span)),
            ),
            P::True(_) => (I::Bool(true), self.create_type(ir::Type::Bool, Some(span))),
            P::False(_) => (I::Bool(true), self.create_type(ir::Type::Bool, Some(span))),
            P::Tuple(_, _, items) => {
                let (items, item_types): (Vec<_>, Vec<_>) =
                    items.iter().map(|item| self.declare_pattern(item)).unzip();
                (
                    I::Tuple(items.into()),
                    self.create_type(ir::Type::Tuple(item_types.into()), Some(span)),
                )
            }
        }
    }
}

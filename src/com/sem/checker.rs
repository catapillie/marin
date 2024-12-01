use super::Scope;
use crate::com::{
    ast::{self},
    ir,
    reporting::{Header, Label, Report},
};

pub struct Checker<'src, 'e> {
    source: &'src str,
    file: usize,
    reports: &'e mut Vec<Report>,

    scope: Scope<'src, ()>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(source: &'src str, file: usize, reports: &'e mut Vec<Report>) -> Self {
        Self {
            source,
            file,
            reports,

            scope: Scope::root(),
        }
    }

    fn open_scope(&mut self) {
        self.scope.open(false);
    }

    fn close_scope(&mut self) {
        self.scope.close();
    }

    pub fn check_file(&mut self, ast: &ast::File) {
        for expr in &ast.0 {
            self.check_expression(expr);
        }
    }

    fn check_statement(&mut self, e: &ast::Expr) -> ir::Stmt {
        use ast::Expr as E;
        match e {
            E::Let(..) => todo!(),
            E::Import(..) => todo!(),
            _ => ir::Stmt::Expr(self.check_expression(e)),
        }
    }

    fn check_expression(&mut self, e: &ast::Expr) -> ir::Expr {
        use ast::Expr as E;
        match e {
            E::Missing(e) => self.check_missing(e),
            E::Int(e) => self.check_int(e),
            E::Float(e) => self.check_float(e),
            E::String(e) => self.check_string(e),
            E::True(e) => self.check_bool(e, true),
            E::False(e) => self.check_bool(e, false),
            E::Var(..) => todo!(),
            E::Tuple(e) => self.check_tuple(e),
            E::Array(e) => self.check_array(e),
            E::Block(e) => self.check_block(e),
            E::Loop(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Call(..) => todo!(),
            E::Access(..) => todo!(),
            E::Fun(..) => todo!(),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                ir::Expr::Missing
            }
        }
    }

    fn check_missing(&mut self, _: &ast::Lexeme) -> ir::Expr {
        ir::Expr::Missing
    }

    fn check_int(&mut self, e: &ast::Lexeme) -> ir::Expr {
        let lexeme = e.span.lexeme(self.source);
        match lexeme.parse::<i64>() {
            Ok(n) => ir::Expr::Int(n),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidInteger())
                        .with_primary_label(Label::Empty, e.span.wrap(self.file)),
                );
                ir::Expr::Missing
            }
        }
    }

    fn check_float(&mut self, e: &ast::Lexeme) -> ir::Expr {
        let lexeme = e.span.lexeme(self.source);
        match lexeme.parse::<f64>() {
            Ok(f) => ir::Expr::Float(f),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidFloat())
                        .with_primary_label(Label::Empty, e.span.wrap(self.file)),
                );
                ir::Expr::Missing
            }
        }
    }

    fn check_string(&mut self, e: &ast::Lexeme) -> ir::Expr {
        let lit = &self.source[(e.span.start + 1)..(e.span.end - 1)];
        ir::Expr::String(lit.to_string())
    }

    fn check_bool(&mut self, _: &ast::Lexeme, b: bool) -> ir::Expr {
        ir::Expr::Bool(b)
    }

    fn check_tuple(&mut self, e: &ast::Tuple) -> ir::Expr {
        if e.items.len() == 1 {
            return self.check_expression(&e.items[0]);
        }

        ir::Expr::Tuple(e.items.iter().map(|e| self.check_expression(e)).collect())
    }

    fn check_array(&mut self, e: &ast::Array) -> ir::Expr {
        ir::Expr::Array(e.items.iter().map(|e| self.check_expression(e)).collect())
    }

    fn check_block(&mut self, e: &ast::Block) -> ir::Expr {
        let mut iter = e.items.iter().peekable();
        let mut stmts = Vec::with_capacity(e.items.len());
        let mut last = None;
        
        self.open_scope();
        
        while let Some(item) = iter.next() {
            let s = self.check_statement(item);
            if iter.peek().is_none() {
                if let ir::Stmt::Expr(e) = s {
                    last = Some(e)
                }
                continue;
            }
            stmts.push(s);
        }

        self.close_scope();

        let last = last.unwrap_or(ir::Expr::unit());
        ir::Expr::Block(stmts.into(), Box::new(last))
    }
}

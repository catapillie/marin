use super::{Error, Value};
use crate::com::{ir, scope::Scope};

type Result<T> = std::result::Result<T, Error>;

pub struct Walker<'a> {
    variables: Scope<'a, Value>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            variables: Scope::root(),
        }
    }

    pub fn eval_file(&mut self, ir: &ir::File) -> Result<()> {
        for stmt in &ir.stmts {
            self.eval_statement(stmt)?;
        }
        Ok(())
    }

    pub fn eval_statement(&mut self, stmt: &ir::Stmt) -> Result<()> {
        use ir::Stmt as S;
        match stmt {
            S::Expr(e, _) => {
                let value = self.eval_expression(e)?;
                eprintln!("{value:?}");
                Ok(())
            }
            S::Let => todo!(),
        }
    }

    fn eval_expression(&mut self, e: &ir::Expr) -> Result<Value> {
        use ir::Expr as E;
        match e {
            E::Missing => Err(Error::Missing),
            E::Int(n) => Ok(Value::Int(*n)),
            E::Float(f) => Ok(Value::Float(*f)),
            E::String(s) => Ok(Value::String(s.clone())),
            E::Bool(b) => Ok(Value::Bool(*b)),
            E::Var(id) => self.eval_var(*id),
            E::Tuple(items) => self.eval_tuple(items),
            E::Array(items) => self.eval_array(items),
            E::Block(stmts, expr) => self.eval_block(stmts, expr),
        }
    }

    fn eval_var(&self, _: ir::EntityID) -> Result<Value> {
        todo!()
    }

    fn eval_tuple(&mut self, items: &[ir::Expr]) -> Result<Value> {
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(self.eval_expression(item)?);
        }
        Ok(Value::Tuple(values.into()))
    }

    fn eval_array(&mut self, items: &[ir::Expr]) -> Result<Value> {
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(self.eval_expression(item)?);
        }
        Ok(Value::Array(values.into()))
    }

    fn eval_block(&mut self, stmts: &[ir::Stmt], expr: &ir::Expr) -> Result<Value> {
        for stmt in stmts {
            self.eval_statement(stmt)?;
        }
        self.eval_expression(expr)
    }
}

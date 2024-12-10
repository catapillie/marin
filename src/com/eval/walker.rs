use super::{Error, Value};
use crate::com::{ir, scope::Scope};

type Result<T> = std::result::Result<T, State>;

pub struct Walker<'a> {
    variables: Scope<'a, Value>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            variables: Scope::root(),
        }
    }

    pub fn eval_file(&mut self, ir: &ir::File) -> std::result::Result<(), Error> {
        for stmt in &ir.stmts {
            if let Some(e) = self.eval_statement(stmt)? {
                println!("{e}")
            }
        }
        Ok(())
    }

    fn eval_statement(&mut self, stmt: &ir::Stmt) -> Result<Option<Value>> {
        use ir::Stmt as S;
        match stmt {
            S::Expr(e, _) => self.eval_expression(e).map(Some),
            S::Let => todo!(),
        }
    }

    fn eval_expression(&mut self, e: &ir::Expr) -> Result<Value> {
        use ir::Expr as E;
        match e {
            E::Missing => Err(State::Error(Error::Missing)),
            E::Int(n) => Ok(Value::Int(*n)),
            E::Float(f) => Ok(Value::Float(*f)),
            E::String(s) => Ok(Value::String(s.clone())),
            E::Bool(b) => Ok(Value::Bool(*b)),
            E::Var(id) => self.eval_var(*id),
            E::Tuple(items) => self.eval_tuple(items),
            E::Array(items) => self.eval_array(items),
            E::Block(stmts, id) => self.eval_block(stmts, id.0),
            E::Conditional(branches, exhaustive) => self.eval_conditional(branches, *exhaustive),
            E::Break(None, id) => self.eval_break(id.0),
            E::Break(Some(value), id) => self.eval_break_with(value, id.0),
            E::Skip(id) => self.eval_skip(id.0),
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

    fn eval_block(&mut self, stmts: &[ir::Stmt], label_id: usize) -> Result<Value> {
        let mut last = Value::unit();
        for stmt in stmts {
            match self.eval_statement(stmt) {
                Err(State::Break(id)) if label_id == id => return Ok(Value::unit()),
                Err(State::BreakWith(id, val)) if label_id == id => return Ok(val),
                Err(e) => return Err(e),
                Ok(Some(val)) => last = val,
                Ok(_) => continue,
            }
        }

        Ok(last)
    }

    fn eval_conditional(&mut self, branches: &[ir::Branch], is_exhaustive: bool) -> Result<Value> {
        for branch in branches {
            if let Some(val) = self.eval_branch(branch)? {
                if is_exhaustive {
                    return Ok(val);
                } else {
                    return Ok(Value::unit());
                }
            }
        }

        Err(State::Error(Error::InvalidState))
    }

    fn eval_branch(&mut self, b: &ir::Branch) -> Result<Option<Value>> {
        use ir::Branch as B;
        match b {
            B::If(condition, stmts, label_id) => self.eval_if(condition, stmts, label_id.0),
            B::While(condition, stmts, label_id) => self.eval_while(condition, stmts, label_id.0),
            B::Loop(stmts, label_id) => self.eval_loop(stmts, label_id.0),
            B::Else(stmts, label_id) => self.eval_else(stmts, label_id.0),
        }
    }

    fn eval_if(
        &mut self,
        condition: &ir::Expr,
        stmts: &[ir::Stmt],
        label_id: usize,
    ) -> Result<Option<Value>> {
        match self.eval_expression(condition)? {
            Value::Bool(true) => {}
            Value::Bool(false) => return Ok(None),
            _ => return Err(State::Error(Error::NonBooleanCondition)),
        };

        let mut last = Value::unit();
        for stmt in stmts {
            match self.eval_statement(stmt) {
                Err(State::Break(id)) if label_id == id => return Ok(Some(Value::unit())),
                Err(State::BreakWith(id, val)) if label_id == id => return Ok(Some(val)),
                Err(e) => return Err(e),
                Ok(Some(val)) => last = val,
                Ok(_) => continue,
            }
        }

        Ok(Some(last))
    }

    fn eval_while(
        &mut self,
        condition: &ir::Expr,
        stmts: &[ir::Stmt],
        label_id: usize,
    ) -> Result<Option<Value>> {
        loop {
            match self.eval_expression(condition)? {
                Value::Bool(true) => {}
                Value::Bool(false) => return Ok(None),
                _ => return Err(State::Error(Error::NonBooleanCondition)),
            };

            for stmt in stmts {
                match self.eval_statement(stmt) {
                    Err(State::Break(id)) if label_id == id => return Ok(Some(Value::unit())),
                    Err(State::BreakWith(id, val)) if label_id == id => return Ok(Some(val)),
                    Err(State::Skip(id)) if label_id == id => break,
                    Err(e) => return Err(e),
                    Ok(_) => continue,
                }
            }
        }
    }

    fn eval_loop(&mut self, stmts: &[ir::Stmt], label_id: usize) -> Result<Option<Value>> {
        loop {
            for stmt in stmts {
                match self.eval_statement(stmt) {
                    Err(State::Break(id)) if label_id == id => return Ok(Some(Value::unit())),
                    Err(State::BreakWith(id, val)) if label_id == id => return Ok(Some(val)),
                    Err(State::Skip(id)) if label_id == id => break,
                    Err(e) => return Err(e),
                    Ok(_) => continue,
                }
            }
        }
    }

    fn eval_else(&mut self, stmts: &[ir::Stmt], label_id: usize) -> Result<Option<Value>> {
        self.eval_block(stmts, label_id).map(Some)
    }

    fn eval_break(&mut self, id: usize) -> Result<Value> {
        Err(State::Break(id))
    }

    fn eval_break_with(&mut self, value: &ir::Expr, id: usize) -> Result<Value> {
        Err(State::BreakWith(id, self.eval_expression(value)?))
    }

    fn eval_skip(&mut self, id: usize) -> Result<Value> {
        Err(State::Skip(id))
    }
}

#[derive(Debug)]
enum State {
    Error(Error),
    Break(usize),
    BreakWith(usize, Value),
    Skip(usize),
}

impl From<State> for Error {
    fn from(state: State) -> Self {
        match state {
            State::Error(error) => error,
            _ => Error::InvalidState,
        }
    }
}

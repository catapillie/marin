use super::{Error, Value};
use crate::com::{ir, scope::Scope};

type Result<'a, T> = std::result::Result<T, State<'a>>;

pub struct Walker<'a> {
    variables: Scope<usize, (), Value<'a>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            variables: Scope::root(),
        }
    }

    pub fn eval_file(&mut self, ir: &'a ir::File) -> std::result::Result<(), Error> {
        for stmt in &ir.stmts {
            if let Some(e) = self.eval_statement(stmt)? {
                println!("{e}")
            }
        }
        Ok(())
    }

    fn eval_statement(&mut self, stmt: &'a ir::Stmt) -> Result<'a, Option<Value<'a>>> {
        use ir::Stmt as S;
        match stmt {
            S::Missing => Err(State::Error(Error::InvalidState)),
            S::Nothing => Ok(None),
            S::Expr(e, _) => self.eval_expression(e).map(Some),
            S::Let(pattern, value) => {
                self.eval_let(pattern, value)?;
                Ok(None)
            }
        }
    }

    fn eval_let(&mut self, p: &'a ir::Pattern, e: &'a ir::Expr) -> Result<'a, ()> {
        let value = self.eval_expression(e)?;
        self.deconstruct(p, value)?;
        Ok(())
    }

    fn deconstruct(&mut self, p: &'a ir::Pattern, v: Value<'a>) -> Result<'a, bool> {
        use ir::Pattern as P;
        use Value as V;
        match (p, v) {
            (P::Missing, _) => Err(State::Error(Error::Missing)),

            (P::Discard, _) => Ok(true),
            (P::Binding(id), v) => {
                self.variables.insert(id.0, v);
                Ok(true)
            }

            (P::Int(a), V::Int(b)) => Ok(*a == b),
            (P::Float(a), V::Float(b)) => Ok(*a == b),
            (P::String(a), V::String(b)) => Ok(a == &b),
            (P::Bool(a), V::Bool(b)) => Ok(*a == b),

            (P::Tuple(left_items), V::Tuple(right_items))
                if left_items.len() == right_items.len() =>
            {
                let mut matched = true;
                for (left, right) in left_items.iter().zip(right_items) {
                    matched &= self.deconstruct(left, right)?;
                }
                Ok(matched)
            }

            (
                P::Variant(_, tag_left, Some(left_items)),
                V::Variant(tag_right, Some(right_items)),
            ) => {
                if *tag_left != tag_right || left_items.len() != right_items.len() {
                    return Ok(false);
                }

                let mut matched = true;
                for (left, right) in left_items.iter().zip(right_items) {
                    matched &= self.deconstruct(left, right)?;
                }
                Ok(matched)
            }
            (P::Variant(_, tag_left, _), V::Variant(tag_right, _)) => Ok(*tag_left == tag_right),

            (P::Record(_, left_fields), V::Record(right_fields)) => {
                let mut matched = true;
                for (left, right) in left_fields.iter().zip(right_fields) {
                    matched &= self.deconstruct(left, right)?;
                }
                Ok(matched)
            }

            _ => Err(State::Error(Error::PatternMismatch)),
        }
    }

    fn eval_expression(&mut self, e: &'a ir::Expr) -> Result<'a, Value<'a>> {
        use ir::Expr as E;
        match e {
            E::Missing => Err(State::Error(Error::Missing)),
            E::Int(n) => Ok(Value::Int(*n)),
            E::Float(f) => Ok(Value::Float(*f)),
            E::String(s) => Ok(Value::String(s.clone())),
            E::Bool(b) => Ok(Value::Bool(*b)),
            E::Var(id) => self.eval_var(id.0),
            E::Tuple(items) => self.eval_tuple(items),
            E::Array(items) => self.eval_array(items),
            E::Block(stmts, id) => self.eval_block(stmts, id.0),
            E::Conditional(branches, exhaustive) => self.eval_conditional(branches, *exhaustive),
            E::Break(None, id) => self.eval_break(id.0),
            E::Break(Some(value), id) => self.eval_break_with(value, id.0),
            E::Skip(id) => self.eval_skip(id.0),
            E::Fun(rec_id, sig, value) => self.eval_fun(sig, value, *rec_id),
            E::Call(callee, args) => self.eval_call(callee, args),
            E::Variant(tag, items) => self.eval_variant(*tag, items),
            E::Record(values) => self.eval_record(values),
            E::Access(rec, tag) => self.eval_access(rec, *tag),
        }
    }

    fn eval_var(&self, id: usize) -> Result<'a, Value<'a>> {
        self.variables
            .search(id)
            .cloned()
            .ok_or(State::Error(Error::UnknownVariable))
    }

    fn eval_tuple(&mut self, items: &'a [ir::Expr]) -> Result<'a, Value<'a>> {
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(self.eval_expression(item)?);
        }
        Ok(Value::Tuple(values.into()))
    }

    fn eval_array(&mut self, items: &'a [ir::Expr]) -> Result<'a, Value<'a>> {
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(self.eval_expression(item)?);
        }
        Ok(Value::Array(values.into()))
    }

    fn eval_block(&mut self, stmts: &'a [ir::Stmt], label_id: usize) -> Result<'a, Value<'a>> {
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

    fn eval_conditional(
        &mut self,
        branches: &'a [ir::Branch],
        is_exhaustive: bool,
    ) -> Result<'a, Value<'a>> {
        for branch in branches {
            if let Some(val) = self.eval_branch(branch)? {
                if is_exhaustive {
                    return Ok(val);
                } else {
                    return Ok(Value::unit());
                }
            }
        }

        match is_exhaustive {
            true => Err(State::Error(Error::InvalidState)),
            false => Ok(Value::unit()),
        }
    }

    fn eval_branch(&mut self, b: &'a ir::Branch) -> Result<'a, Option<Value<'a>>> {
        use ir::Branch as B;
        match b {
            B::If(condition, stmts, label_id) => self.eval_if(condition, stmts, label_id.0),
            B::While(condition, stmts, label_id) => self.eval_while(condition, stmts, label_id.0),
            B::Loop(stmts, label_id) => self.eval_loop(stmts, label_id.0),
            B::Else(stmts, label_id) => self.eval_else(stmts, label_id.0),
            B::Match(id, scrutinee, decision) => self.eval_match(id.0, scrutinee, decision),
        }
    }

    fn eval_if(
        &mut self,
        condition: &'a ir::Expr,
        stmts: &'a [ir::Stmt],
        label_id: usize,
    ) -> Result<'a, Option<Value<'a>>> {
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
        condition: &'a ir::Expr,
        stmts: &'a [ir::Stmt],
        label_id: usize,
    ) -> Result<'a, Option<Value<'a>>> {
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

    fn eval_loop(
        &mut self,
        stmts: &'a [ir::Stmt],
        label_id: usize,
    ) -> Result<'a, Option<Value<'a>>> {
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

    fn eval_else(
        &mut self,
        stmts: &'a [ir::Stmt],
        label_id: usize,
    ) -> Result<'a, Option<Value<'a>>> {
        self.eval_block(stmts, label_id).map(Some)
    }

    fn eval_match(
        &mut self,
        id: usize,
        scrutinee: &'a ir::Expr,
        decision: &'a ir::Decision,
    ) -> Result<'a, Option<Value<'a>>> {
        let scrutinee = self.eval_expression(scrutinee)?;
        self.variables.insert(id, scrutinee);
        self.eval_decision(decision)
    }

    fn eval_decision(&mut self, decision: &'a ir::Decision) -> Result<'a, Option<Value<'a>>> {
        use ir::Decision as D;
        match decision {
            D::Failure => Ok(None),
            D::Success(stmts, expr) => {
                for stmt in stmts {
                    self.eval_statement(stmt)?;
                }
                Ok(Some(self.eval_expression(expr)?))
            }
            D::Test(id, pat, success, failure) => {
                let value = self.eval_var(id.0)?;
                match self.deconstruct(pat, value)? {
                    true => self.eval_decision(success),
                    false => self.eval_decision(failure),
                }
            }
        }
    }

    fn eval_break(&mut self, id: usize) -> Result<'a, Value<'a>> {
        Err(State::Break(id))
    }

    fn eval_break_with(&mut self, value: &'a ir::Expr, id: usize) -> Result<'a, Value<'a>> {
        Err(State::BreakWith(id, self.eval_expression(value)?))
    }

    fn eval_skip(&mut self, id: usize) -> Result<'a, Value<'a>> {
        Err(State::Skip(id))
    }

    fn eval_fun(
        &self,
        sig: &'a ir::Signature,
        value: &'a ir::Expr,
        rec_id: Option<ir::EntityID>,
    ) -> Result<'a, Value<'a>> {
        Ok(Value::Lambda(Vec::new(), sig, value, rec_id))
    }

    fn eval_call(&mut self, callee: &'a ir::Expr, args: &'a [ir::Expr]) -> Result<'a, Value<'a>> {
        let fun = self.eval_expression(callee)?;

        use ir::Signature as S;
        let Value::Lambda(mut provided, sig @ S::Args(fun_args, next_sig), value, id) = fun else {
            return Err(State::Error(Error::InvalidFunction));
        };

        if let Some(id) = id {
            self.variables
                .insert(id.0, Value::Lambda(vec![], sig, value, Some(id)));
        }

        if fun_args.len() != args.len() {
            return Err(State::Error(Error::InvalidArgCount));
        }

        for (arg_pattern, arg) in fun_args.iter().zip(args) {
            provided.push((arg_pattern, self.eval_expression(arg)?));
        }

        let result = match &**next_sig {
            S::Missing => return Err(State::Error(Error::InvalidFunction)),
            S::Args(_, _) => Value::Lambda(provided, next_sig, value, None),
            S::Done => {
                self.variables.open(false);

                for (arg_pattern, arg) in provided {
                    self.deconstruct(arg_pattern, arg)?;
                }
                let result = match self.eval_expression(value) {
                    Ok(val) => val,
                    Err(s) => return Err(State::Error(s.into())),
                };

                self.variables.close();
                result
            }
        };

        Ok(result)
    }

    fn eval_variant(
        &mut self,
        tag: usize,
        items: &'a Option<Box<[ir::Expr]>>,
    ) -> Result<'a, Value<'a>> {
        let items = match items {
            Some(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in &**items {
                    values.push(self.eval_expression(item)?);
                }
                Some(values.into_boxed_slice())
            }
            None => None,
        };
        Ok(Value::Variant(tag, items))
    }

    fn eval_record(&mut self, fields: &'a [ir::Expr]) -> Result<'a, Value<'a>> {
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            values.push(self.eval_expression(field)?);
        }
        Ok(Value::Record(values.into()))
    }

    fn eval_access(&mut self, rec: &'a ir::Expr, tag: usize) -> Result<'a, Value<'a>> {
        let rec = self.eval_expression(rec)?;
        match rec {
            Value::Record(items) => Ok(items[tag].clone()),
            _ => Err(State::Error(Error::InvalidState)),
        }
    }
}

#[derive(Debug)]
enum State<'a> {
    Error(Error),
    Break(usize),
    BreakWith(usize, Value<'a>),
    Skip(usize),
}

impl<'a> From<State<'a>> for Error {
    fn from(state: State) -> Self {
        match state {
            State::Error(error) => error,
            _ => Error::InvalidState,
        }
    }
}

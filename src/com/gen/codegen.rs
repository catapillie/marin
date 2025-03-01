use byteorder::{WriteBytesExt, LE};
use std::{
    collections::HashMap,
    io::{self, Cursor},
};

use crate::{
    binary::{self, opcode, Opcode},
    com::ir,
    exe,
};

struct Function {
    pos: Option<u32>,
    name: String,
    placeholders: Vec<u32>,
}

struct Frame {
    local_count: usize,
}

enum FunctionCode<'a> {
    Expr(&'a ir::Expr),
    Block(&'a [ir::Stmt]),
}

struct FunctionWork<'a> {
    id: usize,
    signature: Option<&'a ir::Signature>,
    body: FunctionCode<'a>,
}

pub struct Codegen<'a> {
    ir: &'a [ir::Module],
    entities: Vec<ir::Entity>,

    constants: Vec<exe::Value>,
    functions: Vec<Function>,

    cursor: Cursor<Vec<u8>>,
    frames: Vec<Frame>,
    local_index: u8,
    locals_by_id: HashMap<ir::EntityID, u8>,
    remaining_work: Vec<FunctionWork<'a>>,
    functions_by_id: HashMap<ir::EntityID, usize>,
}

impl<'a> Codegen<'a> {
    pub fn new(ir: &'a [ir::Module], entities: Vec<ir::Entity>) -> Self {
        Self {
            ir,
            entities,

            constants: Vec::new(),
            functions: Vec::new(),

            cursor: Cursor::new(Vec::new()),
            frames: Vec::new(),
            local_index: 0,
            locals_by_id: HashMap::new(),
            remaining_work: Vec::new(),
            functions_by_id: HashMap::new(),
        }
    }

    fn pos(&self) -> u32 {
        self.cursor
            .position()
            .try_into()
            .expect("current cursor position is too large")
    }

    fn set_pos(&mut self, pos: u32) {
        self.cursor.set_position(pos as u64);
    }

    fn write_u32_placeholder(&mut self) -> io::Result<u32> {
        let pos = self.pos();
        self.cursor.write_u32::<LE>(0)?;
        Ok(pos)
    }

    fn patch_u32_placeholder(&mut self, pos: u32, val: u32) -> io::Result<()> {
        let orig = self.pos();
        self.set_pos(pos);
        self.cursor.write_u32::<LE>(val)?;
        self.set_pos(orig);
        Ok(())
    }

    fn add_constant(&mut self, value: exe::Value) -> u16 {
        let id: u16 = self
            .constants
            .len()
            .try_into()
            .expect("program contains too many constants");
        self.constants.push(value);
        id
    }

    fn write_opcode(&mut self, opcode: &Opcode) -> binary::Result<()> {
        binary::write_opcode(&mut self.cursor, opcode)
    }

    fn create_function(
        &mut self,
        name: &str,
        signature: Option<&'a ir::Signature>,
        body: FunctionCode<'a>,
    ) -> usize {
        let id = self.functions.len();
        self.functions.push(Function {
            pos: None,
            name: name.to_string(),
            placeholders: Vec::new(),
        });
        self.remaining_work.push(FunctionWork {
            id,
            signature,
            body,
        });
        id
    }

    pub fn gen(&mut self) -> binary::Result<()> {
        // initial function
        for (i, ir) in self.ir.iter().enumerate() {
            let name = format!("<main{i}>");
            self.create_function(&name, None, FunctionCode::Block(&ir.stmts));
        }

        // generate all functions
        while let Some(work) = self.remaining_work.pop() {
            self.gen_function(work)?;
        }

        Ok(())
    }

    fn gen_function(&mut self, work: FunctionWork<'a>) -> binary::Result<()> {
        // reset local counter and registered locals
        self.local_index = 0;
        self.locals_by_id.clear();

        // update function's code position
        let pos = self.pos();
        let fun = self
            .functions
            .get_mut(work.id)
            .expect("generating an unregistered function");
        fun.pos = Some(pos);
        for placeholder in fun.placeholders.clone() {
            self.patch_u32_placeholder(placeholder, pos)?;
        }

        use ir::Signature as Sig;
        if let Some(sig) = work.signature {
            let Sig::Args(args, next) = sig else {
                unreachable!("invalid function signature");
            };

            let Sig::Done = &**next else {
                panic!("unhandled higher order functions");
            };

            // account for parameters already living on the stack
            let arg_count: u8 = args
                .len()
                .try_into()
                .expect("function has more than 255 arguments");
            self.local_index += arg_count;

            // deconstruct each one of them
            for (i, arg) in args.iter().enumerate() {
                let local_id = i as u8;
                self.gen_register_pattern_locals(arg)?;
                self.write_opcode(&Opcode::load_local(local_id))?;
                self.gen_initialize_pattern(arg)?;
            }
        }

        match work.body {
            FunctionCode::Expr(expr) => self.gen_expression(expr)?,
            FunctionCode::Block(stmts) => self.gen_expression_block(stmts)?,
        }

        self.write_opcode(&Opcode::ret)?;
        Ok(())
    }

    fn gen_statement(&mut self, stmt: &'a ir::Stmt) -> binary::Result<()> {
        use ir::Stmt as S;
        match stmt {
            S::Missing => Ok(()),
            S::Nothing => Ok(()),
            S::Expr(expr, _) => {
                self.gen_expression(expr)?;
                self.write_opcode(&Opcode::pop)?;
                Ok(())
            }
            S::Let(pattern, expr) => {
                self.gen_deconstruct(pattern, expr)?;
                Ok(())
            }
        }
    }

    fn gen_deconstruct(
        &mut self,
        pattern: &'a ir::Pattern,
        expr: &'a ir::Expr,
    ) -> binary::Result<()> {
        self.gen_register_pattern_locals(pattern)?;
        self.gen_expression(expr)?;
        self.gen_initialize_pattern(pattern)?;
        Ok(())
    }

    fn gen_register_pattern_locals(&mut self, pattern: &'a ir::Pattern) -> binary::Result<()> {
        use ir::Pattern as P;
        match pattern {
            P::Missing => Ok(()),
            P::Discard => Ok(()),
            P::Binding(id) => {
                self.write_opcode(&Opcode::load_nil)?;
                let local = self.register_local();
                self.locals_by_id.insert(*id, local);
                Ok(())
            }
            P::Int(_) | P::Float(_) | P::String(_) | P::Bool(_) => Ok(()),
            P::Tuple(items) => {
                for item in items {
                    self.gen_register_pattern_locals(item)?;
                }
                Ok(())
            }
            P::Variant(_, _, _) => todo!(),
            P::Record(_, fields) => {
                for field in fields {
                    self.gen_register_pattern_locals(field)?;
                }
                Ok(())
            }
        }
    }

    // initializes the pattern with the expression on the stack
    fn gen_initialize_pattern(&mut self, pattern: &'a ir::Pattern) -> binary::Result<()> {
        use ir::Pattern as P;
        match pattern {
            P::Missing => Ok(()),
            P::Discard => {
                self.write_opcode(&Opcode::pop)?;
                Ok(())
            }
            P::Binding(id) => {
                let local = self.get_local(id);
                self.write_opcode(&Opcode::set_local(local))?;
                Ok(())
            }
            P::Int(_) | P::Float(_) | P::String(_) | P::Bool(_) => Ok(()),
            P::Tuple(items) => {
                for (i, item) in items.iter().enumerate() {
                    self.write_opcode(&Opcode::index(i as u8))?;
                    self.gen_initialize_pattern(item)?;
                }
                self.write_opcode(&Opcode::pop)?;
                Ok(())
            }
            P::Variant(_, _, _) => todo!(),
            P::Record(_, fields) => {
                for (i, field) in fields.iter().enumerate() {
                    self.write_opcode(&Opcode::index(i as u8))?;
                    self.gen_initialize_pattern(field)?;
                }
                self.write_opcode(&Opcode::pop)?;
                Ok(())
            }
        }
    }

    fn open_frame(&mut self) -> binary::Result<()> {
        self.write_opcode(&Opcode::do_frame)?;
        self.frames.push(Frame { local_count: 0 });
        Ok(())
    }

    fn close_frame(&mut self) -> binary::Result<()> {
        let frame = self.frames.pop().expect("frame stack underflow");
        self.local_index -= frame.local_count as u8;

        self.write_opcode(&Opcode::end_frame)?;
        Ok(())
    }

    fn register_local(&mut self) -> u8 {
        let index = self.local_index;
        self.local_index = self
            .local_index
            .checked_add(1)
            .expect("exceeded maximum local count (256) in function");
        index
    }

    fn get_local(&self, id: &ir::EntityID) -> u8 {
        self.locals_by_id
            .get(id)
            .copied()
            .expect("unregistered local")
    }

    fn register_user_function(
        &mut self,
        id: Option<ir::EntityID>,
        name: &str,
        signature: Option<&'a ir::Signature>,
        body: FunctionCode<'a>,
    ) -> usize {
        let fun_id = self.create_function(name, signature, body);

        if let Some(id) = id {
            if self.functions_by_id.contains_key(&id) {
                panic!("function already registered");
            }
            self.functions_by_id.insert(id, fun_id);
        }

        fun_id
    }

    fn try_get_user_function(&self, id: &ir::EntityID) -> Option<usize> {
        self.functions_by_id.get(id).copied()
    }

    fn gen_statement_block(&mut self, stmts: &'a [ir::Stmt]) -> binary::Result<()> {
        self.open_frame()?;
        for stmt in stmts {
            self.gen_statement(stmt)?;
        }
        self.close_frame()?;
        Ok(())
    }

    fn gen_expression_block(&mut self, stmts: &'a [ir::Stmt]) -> binary::Result<()> {
        use ir::Stmt as S;
        let stmts = stmts
            .iter()
            .filter(|stmt| match stmt {
                S::Missing => false,
                S::Nothing => true,
                S::Expr(..) => true,
                S::Let(..) => true,
            })
            .collect::<Vec<_>>();

        let Some(last) = stmts.last() else {
            return self.gen_unit();
        };

        self.open_frame()?;
        match last {
            S::Expr(expr, _) => {
                for stmt in &stmts[..stmts.len() - 1] {
                    self.gen_statement(stmt)?;
                }
                self.gen_expression(expr)?;
            }
            _ => {
                for stmt in stmts {
                    self.gen_statement(stmt)?;
                }
                self.gen_unit()?;
            }
        }
        self.close_frame()?;

        Ok(())
    }

    fn gen_unit(&mut self) -> binary::Result<()> {
        self.write_opcode(&Opcode::bundle(0))?;
        Ok(())
    }

    fn gen_expression(&mut self, expr: &'a ir::Expr) -> binary::Result<()> {
        use ir::Expr as E;
        match expr {
            E::Missing => Ok(()),
            E::Int(n) => {
                let id = self.add_constant(exe::Value::Int(*n));
                self.write_opcode(&Opcode::load_const(id))?;
                Ok(())
            }
            E::Float(f) => {
                let id = self.add_constant(exe::Value::Float(*f));
                self.write_opcode(&Opcode::load_const(id))?;
                Ok(())
            }
            E::String(s) => {
                let id = self.add_constant(exe::Value::String(s.clone()));
                self.write_opcode(&Opcode::load_const(id))?;
                Ok(())
            }
            E::Bool(b) => {
                let id = self.add_constant(exe::Value::Bool(*b));
                self.write_opcode(&Opcode::load_const(id))?;
                Ok(())
            }
            E::Var(id) => {
                if let Some(fun_id) = self.try_get_user_function(id) {
                    self.gen_function_value(fun_id)?;
                    Ok(())
                } else {
                    let local = self.get_local(id);
                    self.write_opcode(&Opcode::load_local(local))?;
                    Ok(())
                }
            }
            E::Tuple(items) => {
                for item in items {
                    self.gen_expression(item)?;
                }
                let count: u8 = items
                    .len()
                    .try_into()
                    .expect("tuples cannot have more than 255 items");
                self.write_opcode(&Opcode::bundle(count))?;
                Ok(())
            }
            E::Array(..) => todo!(),
            E::Block(stmts, _) => {
                self.gen_expression_block(stmts)?;
                Ok(())
            }
            E::Conditional(branches, is_exhaustive) => {
                let mut else_jump_pos = None;
                let mut exit_jump_pos = Vec::new();

                use ir::Branch as B;
                for branch in branches {
                    if let Some(pos) = else_jump_pos {
                        self.patch_u32_placeholder(pos, self.pos())?;
                    };

                    match branch {
                        B::If(guard, body, _) => {
                            // if <guard> ...
                            self.gen_expression(guard)?;

                            // if false, skip over the 'then' block
                            self.cursor.write_u8(opcode::jump_if_not)?;
                            else_jump_pos = Some(self.write_u32_placeholder()?);

                            // then <body> ...
                            self.gen_expression_block(body)?;
                            self.cursor.write_u8(opcode::jump)?;
                            exit_jump_pos.push(self.write_u32_placeholder()?);
                        }
                        B::While(guard, body, _) => {
                            // while <guard> ...
                            let while_pos = self.pos();
                            self.gen_expression(guard)?;

                            // if false, skip over the 'do' block if
                            self.cursor.write_u8(opcode::jump_if_not)?;
                            else_jump_pos = Some(self.write_u32_placeholder()?);

                            // do <body> ...
                            self.gen_statement_block(body)?;
                            self.write_opcode(&Opcode::jump(while_pos))?;
                        }
                        B::Loop(body, _) => {
                            // loop <body> ...
                            let loop_pos = self.pos();
                            self.gen_statement_block(body)?;
                            self.write_opcode(&Opcode::jump(loop_pos))?;
                        }
                        B::Else(body, _) => {
                            // else <body> ...
                            self.gen_expression_block(body)?;

                            // no need to wire an 'else' because this branch always succeeds
                            else_jump_pos = None;
                            break;
                        }
                        B::Match(..) => todo!(),
                    }
                }

                // wire all exit jumps
                let final_pos = self.pos();
                for exit_pos in exit_jump_pos {
                    self.patch_u32_placeholder(exit_pos, final_pos)?;
                }

                // if non-exhaustive, then all successful branches
                // have to pop: they eventually evaluate to unit down below
                if !*is_exhaustive {
                    self.write_opcode(&Opcode::pop)?;
                }

                // wire final else jump
                if let Some(pos) = else_jump_pos {
                    self.patch_u32_placeholder(pos, self.pos())?;
                };

                // non-exhaustive expressions evaluate to unit
                if !*is_exhaustive {
                    self.gen_unit()?;
                }

                Ok(())
            }
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Fun(name, rec_id, signature, expr) => {
                let id = self.register_user_function(
                    *rec_id,
                    name,
                    Some(signature),
                    FunctionCode::Expr(expr),
                );
                self.gen_function_value(id)?;
                Ok(())
            }
            E::Call(fun, args) => {
                let arg_count: u8 = args
                    .len()
                    .try_into()
                    .expect("call has more than 255 arguemnts");
                for arg in args {
                    self.gen_expression(arg)?;
                }
                self.gen_expression(fun)?;
                self.write_opcode(&Opcode::call(arg_count))?;
                Ok(())
            }
            E::Variant(tag, items) => {
                // gen tag as i64
                let tag_id = self.add_constant(exe::Value::Int(*tag as i64));
                self.write_opcode(&Opcode::load_const(tag_id))?;

                // gen items
                let count: u8 = match items {
                    Some(items) => {
                        for item in items {
                            self.gen_expression(item)?;
                        }
                        items
                            .len()
                            .try_into()
                            .expect("variants cannot have more than 255 items")
                    }
                    None => 0,
                };
                self.write_opcode(&Opcode::bundle(count))?;

                // bundle (tag, [items])
                self.write_opcode(&Opcode::bundle(2))?;
                Ok(())
            }
            E::Record(items) => {
                for item in items {
                    self.gen_expression(item)?;
                }
                let count: u8 = items
                    .len()
                    .try_into()
                    .expect("records cannot have more than 255 fields");
                self.write_opcode(&Opcode::bundle(count))?;
                Ok(())
            }
            E::Access(..) => todo!(),
        }
    }

    fn gen_function_value(&mut self, id: usize) -> binary::Result<()> {
        let fun = &self.functions[id];
        match fun.pos {
            Some(pos) => {
                self.write_opcode(&Opcode::load_fun(pos))?;
            }
            None => {
                self.cursor.write_u8(opcode::load_fun)?;
                let placeholder = self.write_u32_placeholder()?;
                self.functions[id].placeholders.push(placeholder);
            }
        }

        Ok(())
    }

    pub fn done(self) -> binary::Result<Vec<u8>> {
        let function_table = self
            .functions
            .into_iter()
            .map(|f| {
                (
                    f.pos
                        .expect("generated function does not have a code position"),
                    f.name,
                )
            })
            .collect();
        let mut code = self.cursor.into_inner();

        let final_size_approx = code.len() + binary::MAGIC.len() + 1;
        let mut bytecode = Vec::with_capacity(final_size_approx);

        binary::write_magic(&mut bytecode)?;
        binary::write_constant_pool(&mut bytecode, &self.constants)?;
        binary::write_function_table(&mut bytecode, &function_table)?;
        bytecode.append(&mut code);

        Ok(bytecode)
    }
}

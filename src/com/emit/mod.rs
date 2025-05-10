use super::low;
use crate::{
    binary::{self, Opcode},
    exe::{self, Value},
};
use std::{collections::HashMap, io::Cursor};

struct BytecodeBuilder {
    constants: Vec<Value>,
    function_table: HashMap<u32, String>,
    opcodes: Vec<Opcode>,
    cursor: Cursor<Vec<u8>>,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            function_table: HashMap::new(),
            opcodes: Vec::new(),
            cursor: Cursor::new(Vec::new()),
        }
    }

    fn pos(&self) -> u32 {
        self.cursor.position() as u32
    }

    fn write_opcode(&mut self, opcode: Opcode) {
        self.opcodes.push(opcode);
    }

    fn build_program(&mut self, program: low::Program) -> binary::Result<()> {
        for fun in program.functions {
            self.build_function(fun)?;
        }
        Ok(())
    }

    fn build_function(&mut self, function: low::Function) -> binary::Result<()> {
        self.opcodes.clear();

        self.build_expression(function.expr);
        self.write_opcode(Opcode::ret);

        self.function_table.insert(self.pos(), function.name);
        self.emit_bytecode()?;
        Ok(())
    }

    fn build_statement(&mut self, stmt: low::Stmt) {
        use low::Stmt as S;
        match stmt {
            S::Expr { expr } => {
                self.build_expression(*expr);
                self.write_opcode(Opcode::pop);
            }
            S::Let { bindings } => {
                for (pat, expr) in bindings {
                    self.build_expression(expr);
                    self.build_deconstruct(pat, 0);
                }
            }
        }
    }

    fn build_deconstruct(&mut self, pat: low::Pat, at: u16) {
        use low::Pat as P;
        match pat {
            P::Discard | P::Int(_) | P::Float(_) | P::String(_) | P::Bool(_) => {
                self.write_opcode(Opcode::pop_offset(at))
            }
            P::Local(_) => {}
            P::Bundle(items) => {
                self.write_opcode(Opcode::spill(at));
                let size = items.len() as u16;
                for (i, item) in items.into_iter().enumerate() {
                    let offset = size - 1 - i as u16;
                    self.build_deconstruct(item, at + offset);
                }
            }
            P::Variant(_, items) => {
                // get rid of tag
                self.write_opcode(Opcode::spill(at));
                self.write_opcode(Opcode::pop_offset(at + 1));

                // spill & deconstruct inner values
                self.write_opcode(Opcode::spill(at));
                let size = items.len() as u16;
                for (i, item) in items.into_iter().enumerate() {
                    let offset = size - 1 - i as u16;
                    self.build_deconstruct(item, at + offset);
                }
            }
        }
    }

    fn build_expression(&mut self, expr: low::Expr) {
        use low::Expr as E;
        match expr {
            E::Int { val } => self.build_constant(Value::Int(val)),
            E::Float { val } => self.build_constant(Value::Float(val)),
            E::String { val } => self.build_constant(Value::String(val)),
            E::Bool { val } => self.build_constant(Value::Bool(val)),
            E::Bundle { items } => self.build_small_bundle(items),
            E::Block { stmts, result } => {
                for stmt in stmts {
                    self.build_statement(stmt);
                }
                self.build_expression(*result);
            }
            E::Variant { tag, items } => {
                self.build_constant(Value::Int(tag));
                self.build_small_bundle(items);
                self.write_opcode(Opcode::bundle(2));
            }
            E::Local { local } => self.write_opcode(Opcode::load_local(local)),
        }
    }

    fn build_constant(&mut self, value: exe::Value) {
        let index: u16 = match self.constants.iter().position(|v| value.eq(v)) {
            Some(i) => i,
            None => {
                self.constants.push(value);
                self.constants.len() - 1
            }
        }
        .try_into()
        .expect("program cannot have more than 65535 constants");

        self.write_opcode(Opcode::load_const(index));
    }

    fn build_small_bundle(&mut self, items: Box<[low::Expr]>) {
        let size: u8 = items
            .len()
            .try_into()
            .expect("bundle cannot contain more than 255 items");

        for item in items {
            self.build_expression(item);
        }
        self.write_opcode(Opcode::bundle(size));
    }

    fn emit_bytecode(&mut self) -> binary::Result<()> {
        for opcode in &self.opcodes {
            binary::write_opcode(&mut self.cursor, opcode)?;
        }
        Ok(())
    }
}

pub fn emit(program: low::Program) -> binary::Result<Vec<u8>> {
    let mut bb = BytecodeBuilder::new();
    bb.build_program(program)?;

    let mut bytecode = vec![];
    binary::write_magic(&mut bytecode)?;
    binary::write_constant_pool(&mut bytecode, &bb.constants)?;
    binary::write_function_table(&mut bytecode, &bb.function_table)?;
    bytecode.append(&mut bb.cursor.into_inner());

    Ok(bytecode)
}

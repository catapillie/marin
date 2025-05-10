use super::low;
use crate::{
    binary::{self, Opcode},
    exe::{self, Value},
};
use std::io::Cursor;

struct BytecodeBuilder {
    constants: Vec<Value>,

    opcodes: Vec<Opcode>,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            opcodes: Vec::new(),
        }
    }

    fn write_opcode(&mut self, opcode: Opcode) {
        self.opcodes.push(opcode);
    }

    fn build_program(&mut self, program: low::Program) {
        for fun in program.functions {
            self.build_function(fun);
        }
    }

    fn build_function(&mut self, function: low::Function) {
        self.build_expression(*function.expr);
        self.write_opcode(Opcode::ret);
    }

    fn build_statement(&mut self, stmt: low::Stmt) {
        use low::Stmt as S;
        match stmt {
            S::Expr { expr } => {
                self.build_expression(*expr);
                self.write_opcode(Opcode::pop);
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
            E::Tuple { items } => self.build_small_bundle(items),
            E::Array { items } => self.build_small_bundle(items),
            E::Block { stmts, result } => {
                for stmt in stmts {
                    self.build_statement(stmt);
                }
                self.build_expression(*result);
            }
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

    fn emit(self) -> binary::Result<Vec<u8>> {
        let mut cursor = Cursor::new(vec![]);
        binary::write_magic(&mut cursor)?;
        binary::write_constant_pool(&mut cursor, &self.constants)?;
        binary::write_function_table(&mut cursor, &Default::default())?;
        for opcode in self.opcodes {
            binary::write_opcode(&mut cursor, &opcode)?;
        }
        Ok(cursor.into_inner())
    }
}

pub fn emit(program: low::Program) -> binary::Result<Vec<u8>> {
    let mut bb = BytecodeBuilder::new();
    bb.build_program(program);
    bb.emit()
}

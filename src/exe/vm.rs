use super::Value;
use crate::binary::opcode;

#[allow(dead_code)]
#[derive(Clone)]
enum Val {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Bundle(Box<[Val]>),
}

pub struct VM<'a> {
    code: &'a [u8],
    cursor: usize,
    constants: Vec<Val>,
    stack: Vec<Val>,
}

impl<'a> VM<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            code,
            cursor: 0,
            constants: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn to_val(value: &Value) -> Val {
        match value {
            Value::Int(n) => Val::Int(*n),
            Value::Float(f) => Val::Float(*f),
            Value::String(s) => Val::String(s.clone()),
            Value::Bool(b) => Val::Bool(*b),
            Value::Bundle(items) => Val::Bundle((items).into_iter().map(Self::to_val).collect()),
        }
    }

    pub fn add_constant(&mut self, value: &Value) {
        let val = Self::to_val(value);
        self.constants.push(val);
    }

    pub fn run(&mut self) {
        loop {
            let op = self.read_u8();
            match op {
                opcode::bundle => {
                    let count = self.read_u8() as usize;
                    let values = self.stack.split_off(self.stack.len() - count);
                    self.push(Val::Bundle(values.into()));
                }
                opcode::ld_const => {
                    let index = self.read_u16() as usize;
                    self.push(self.constants[index].clone());
                }
                opcode::pop => {
                    self.pop();
                }
                opcode::halt => break,
                _ => panic!("invalid opcode 0x{op:x}"),
            }
        }

        debug_assert!(self.stack.is_empty(), "non-empty stack after halting");
    }

    fn pop(&mut self) -> Val {
        self.stack.pop().expect("stack underflow")
    }

    fn push(&mut self, val: Val) {
        self.stack.push(val);
    }

    fn read_u8(&mut self) -> u8 {
        let x = self.code[self.cursor];
        self.cursor += 1;
        x
    }

    fn read_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.read_u8(), self.read_u8()])
    }
}

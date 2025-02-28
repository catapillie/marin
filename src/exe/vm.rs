use super::Value;
use crate::binary::opcode;

#[allow(dead_code)]
#[derive(Clone)]
enum Val {
    Nil,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Bundle(Box<[Val]>),
}

struct Frame {
    stack_cursor: usize,
}

pub struct VM<'a> {
    code: &'a [u8],
    cursor: usize,
    constants: Vec<Val>,
    stack: Vec<Val>,
    frame_stack: Vec<Frame>,
}

impl<'a> VM<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            code,
            cursor: 0,
            constants: Vec::new(),
            stack: Vec::new(),
            frame_stack: Vec::new(),
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
        // initial frame
        self.push_frame();

        loop {
            let op = self.read_u8();
            match op {
                opcode::bundle => {
                    let count = self.read_u8() as usize;
                    let values = self.stack.split_off(self.stack.len() - count);
                    self.push(Val::Bundle(values.into()));
                }
                opcode::index => {
                    let index = self.read_u8() as usize;
                    let Val::Bundle(items) = self.peek() else {
                        panic!("invalid index on a non-bundle value");
                    };
                    let value = items[index].clone();
                    self.push(value);
                }
                opcode::load_const => {
                    let index = self.read_u16() as usize;
                    self.push(self.constants[index].clone());
                }
                opcode::load_local => {
                    let local = self.read_u8() as usize;
                    let value = self.stack[local].clone();
                    self.push(value);
                }
                opcode::set_local => {
                    let value = self.pop();
                    let local = self.read_u8() as usize;
                    self.stack[local] = value;
                }
                opcode::load_nil => {
                    self.push(Val::Nil);
                }
                opcode::jump => {
                    let pos = self.read_u32() as usize;
                    self.cursor = pos;
                }
                opcode::jump_if => {
                    let pos = self.read_u32() as usize;
                    let Val::Bool(b) = self.pop() else {
                        panic!("found non-boolean value as jump_if condition");
                    };
                    if b {
                        self.cursor = pos;
                    }
                }
                opcode::jump_if_not => {
                    let pos = self.read_u32() as usize;
                    let Val::Bool(b) = self.pop() else {
                        panic!("found non-boolean value as jump_if_not condition");
                    };
                    if !b {
                        self.cursor = pos;
                    }
                }
                opcode::do_frame => {
                    self.push_frame();
                }
                opcode::end_frame => {
                    let value = self.pop();
                    self.pop_frame();
                    self.push(value);
                }
                opcode::ret => {
                    self.pop_frame();
                    if self.frame_stack.is_empty() {
                        break;
                    }
                }
                opcode::pop => {
                    self.pop();
                }
                opcode::dup => {
                    let value = self.peek().clone();
                    self.push(value);
                }
                _ => panic!("invalid opcode 0x{op:x}"),
            }
        }

        debug_assert!(self.stack.is_empty(), "non-empty stack after halting");
    }

    fn pop(&mut self) -> Val {
        self.stack.pop().expect("stack underflow")
    }

    fn peek(&mut self) -> &Val {
        self.stack.last().expect("stack underflow")
    }

    fn push(&mut self, val: Val) {
        self.stack.push(val);
    }

    fn pop_frame(&mut self) {
        let frame = self.frame_stack.pop().expect("frame stack underflow");
        self.stack.truncate(frame.stack_cursor);
    }

    fn push_frame(&mut self) {
        self.frame_stack.push(Frame {
            stack_cursor: self.stack.len(),
        });
    }

    fn read_u8(&mut self) -> u8 {
        let x = self.code[self.cursor];
        self.cursor += 1;
        x
    }

    fn read_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.read_u8(), self.read_u8()])
    }

    fn read_u32(&mut self) -> u32 {
        u32::from_le_bytes([
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
        ])
    }
}

use super::Value;
use crate::binary::opcode;

#[allow(dead_code)]
#[derive(PartialEq, Clone, Debug)]
enum Val {
    Nil,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Func(u32),
    Bundle(Box<[Val]>),
}

struct Frame {
    pos: usize,
    stack_cursor: usize,
    prev_stack_cursor: usize,
}

pub struct VM<'a> {
    code: &'a [u8],
    cursor: usize,
    constants: Vec<Val>,
    stack: Vec<Val>,
    frame_stack: Vec<Frame>,
    frame_cursor: usize,
}

impl<'a> VM<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self {
            code,
            cursor: 0,
            constants: Vec::new(),
            stack: Vec::new(),
            frame_stack: Vec::new(),
            frame_cursor: 0,
        }
    }

    fn to_val(value: &Value) -> Val {
        match value {
            Value::Nil => Val::Nil,
            Value::Int(n) => Val::Int(*n),
            Value::Float(f) => Val::Float(*f),
            Value::String(s) => Val::String(s.clone()),
            Value::Bool(b) => Val::Bool(*b),
            Value::Func => panic!("unallowed user function value"),
            Value::Bundle(items) => Val::Bundle(items.into_iter().map(Self::to_val).collect()),
        }
    }

    fn to_user_val(val: &Val) -> Value {
        match val {
            Val::Nil => Value::Nil,
            Val::Int(n) => Value::Int(*n),
            Val::Float(f) => Value::Float(*f),
            Val::String(s) => Value::String(s.clone()),
            Val::Bool(b) => Value::Bool(*b),
            Val::Func(_) => Value::Func,
            Val::Bundle(items) => Value::Bundle(items.iter().map(Self::to_user_val).collect()),
        }
    }

    pub fn add_constant(&mut self, value: &Value) {
        let val = Self::to_val(value);
        self.constants.push(val);
    }

    pub fn run(&mut self) -> Value {
        // initial frame
        self.push_call_frame(0);

        loop {
            let op = self.read_u8();
            match op {
                opcode::load_fun => {
                    let pos = self.read_u32();
                    self.push(Val::Func(pos));
                }
                opcode::bundle => {
                    let count = self.read_u8() as usize;
                    let values = self.stack.split_off(self.stack.len() - count);
                    self.push(Val::Bundle(values.into()));
                }
                opcode::index_dup => {
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
                    let index = self.frame_cursor + local;
                    let value = self.stack[index].clone();
                    self.push(value);
                }
                opcode::set_local => {
                    let value = self.pop();
                    let local = self.read_u8() as usize;
                    let index = self.frame_cursor + local;
                    self.stack[index] = value;
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
                opcode::jump_eq => {
                    let pos = self.read_u32() as usize;
                    let right = self.pop();
                    let left = self.pop();
                    if left == right {
                        self.cursor = pos;
                    }
                }
                opcode::jump_ne => {
                    let pos = self.read_u32() as usize;
                    let right = self.pop();
                    let left = self.pop();
                    if left != right {
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
                opcode::call => {
                    let arg_count = self.read_u8() as usize;
                    let fun = self.pop();
                    let Val::Func(addr) = fun else {
                        panic!("invalid function object");
                    };
                    self.push_call_frame(arg_count);
                    self.cursor = addr as usize;
                }
                opcode::ret => {
                    let value = self.pop();
                    self.ret_frame();
                    self.push(value);
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

        let result = self.pop();
        debug_assert!(self.stack.is_empty(), "non-empty stack after halting");
        Self::to_user_val(&result)
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

    fn ret_frame(&mut self) {
        let frame = self.frame_stack.pop().expect("frame stack underflow");
        self.stack.truncate(frame.stack_cursor);
        self.cursor = frame.pos;
        self.frame_cursor = frame.prev_stack_cursor;
    }

    fn push_call_frame(&mut self, arg_count: usize) {
        let prev_stack_cursor = self.frame_cursor;
        self.frame_cursor = self.stack.len() - arg_count;
        self.frame_stack.push(Frame {
            pos: self.cursor,
            stack_cursor: self.frame_cursor,
            prev_stack_cursor,
        });
    }

    fn push_frame(&mut self) {
        self.frame_stack.push(Frame {
            pos: self.cursor,
            stack_cursor: self.stack.len(),
            prev_stack_cursor: self.frame_cursor,
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

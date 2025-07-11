use super::Value;
use crate::binary::opcode;

#[derive(PartialEq, Clone, Debug)]
enum Val {
    Nil,
    Int(i64),
    Float(f64),
    String(HeapIndex),
    Bool(bool),
    Func(u32),
    Bundle(HeapIndex),
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
    heap: Heap,
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
            heap: Heap::new(),
            frame_stack: Vec::new(),
            frame_cursor: 0,
        }
    }

    fn fatal(&self, msg: &str) -> ! {
        panic!("FATAL {msg} AT {:0>8}", self.cursor)
    }

    fn store_as_val(&mut self, value: &Value) -> Val {
        match value {
            Value::Nil => Val::Nil,
            Value::Int(n) => Val::Int(*n),
            Value::Float(f) => Val::Float(*f),
            Value::String(s) => Val::String(self.heap.alloc_string(s.clone())),
            Value::Bool(b) => Val::Bool(*b),
            Value::Func => panic!("unallowed user function value"),
            Value::Bundle(items) => {
                let vals = items.iter().map(|item| self.store_as_val(item)).collect();
                Val::Bundle(self.heap.alloc_val_array(vals))
            }
        }
    }

    fn to_user_val(&self, val: &Val) -> Value {
        match val {
            Val::Nil => Value::Nil,
            Val::Int(n) => Value::Int(*n),
            Val::Float(f) => Value::Float(*f),
            Val::String(u) => Value::String(self.heap.deref_string(*u).to_string()),
            Val::Bool(b) => Value::Bool(*b),
            Val::Func(_) => Value::Func,
            Val::Bundle(u) => {
                let values = self
                    .heap
                    .deref_val_array(*u)
                    .iter()
                    .map(|val| self.to_user_val(val))
                    .collect();
                Value::Bundle(values)
            }
        }
    }

    pub fn add_constant(&mut self, value: &Value) {
        let val = self.store_as_val(value);
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
                    let bundle = Val::Bundle(self.heap.alloc_val_array(values));
                    self.push(bundle);
                }
                opcode::bundle_big => {
                    let count = self.read_u64() as usize;
                    let values = self.stack.split_off(self.stack.len() - count);
                    let bundle = Val::Bundle(self.heap.alloc_val_array(values));
                    self.push(bundle);
                }
                opcode::index_dup => {
                    let index = self.read_u8() as usize;
                    let &Val::Bundle(u) = self.peek() else {
                        self.fatal("invalid index on a non-bundle value");
                    };
                    let value = self.heap.deref_val(u, index);
                    self.push(value.clone());
                }
                opcode::index_big_dup => {
                    let index = self.read_u64() as usize;
                    let &Val::Bundle(u) = self.peek() else {
                        self.fatal("invalid index on a non-bundle value");
                    };
                    let value = self.heap.deref_val(u, index);
                    self.push(value.clone());
                }
                opcode::index => {
                    let index = self.read_u8() as usize;
                    let Val::Bundle(u) = self.pop() else {
                        self.fatal("invalid index on a non-bundle value");
                    };
                    let value = self.heap.deref_val(u, index);
                    self.push(value.clone());
                }
                opcode::index_big => {
                    let index = self.read_u64() as usize;
                    let Val::Bundle(u) = self.pop() else {
                        self.fatal("invalid index on a non-bundle value");
                    };
                    let value = self.heap.deref_val(u, index);
                    self.push(value.clone());
                }
                opcode::index_dyn => {
                    let Val::Int(i) = self.pop() else {
                        self.fatal("invalid (dynamic) index with non-integer index")
                    };
                    let Val::Bundle(u) = self.pop() else {
                        self.fatal("invalid (dynamic) index on a non-bundle value");
                    };

                    let value = self.heap.deref_val(u, i as usize);

                    self.push(value.clone());
                }
                opcode::spill => {
                    let offset = self.read_u16() as usize;
                    let index = self.stack.len() - offset - 1;
                    let Val::Bundle(u) = self.stack.remove(index) else {
                        self.fatal("invalid spill on a non-bundle value");
                    };

                    let values = self.heap.deref_val_array(u);
                    self.stack.extend_from_slice(values);
                    self.stack[index..].rotate_right(values.len());
                }
                opcode::add => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a + b),
                        (Val::Float(a), Val::Float(b)) => Val::Float(a + b),
                        (Val::String(u_a), Val::String(u_b)) => {
                            let mut ab = String::new();
                            ab.push_str(self.heap.deref_string(u_a));
                            ab.push_str(self.heap.deref_string(u_b));
                            let u_ab = self.heap.alloc_string(ab);
                            Val::String(u_ab)
                        }
                        _ => self.fatal("invalid 'add' operation"),
                    };
                    self.push(result);
                }
                opcode::sub => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a - b),
                        (Val::Float(a), Val::Float(b)) => Val::Float(a - b),
                        _ => self.fatal("invalid 'sub' operation"),
                    };
                    self.push(result);
                }
                opcode::mul => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a * b),
                        (Val::Float(a), Val::Float(b)) => Val::Float(a * b),
                        _ => self.fatal("invalid 'mul' operation"),
                    };
                    self.push(result);
                }
                opcode::div => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a / b),
                        (Val::Float(a), Val::Float(b)) => Val::Float(a / b),
                        _ => self.fatal("invalid 'mul' operation"),
                    };
                    self.push(result);
                }
                opcode::modulo => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a % b),
                        (Val::Float(a), Val::Float(b)) => Val::Float(a % b),
                        _ => self.fatal("invalid 'modulo' operation"),
                    };
                    self.push(result);
                }
                opcode::pow => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Float(a), Val::Float(b)) => Val::Float(a.powf(b)),
                        _ => self.fatal("invalid 'pow' operation"),
                    };
                    self.push(result);
                }
                opcode::and => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a & b),
                        (Val::Bool(a), Val::Bool(b)) => Val::Bool(a & b),
                        _ => self.fatal("invalid 'and' operation"),
                    };
                    self.push(result);
                }
                opcode::or => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a | b),
                        (Val::Bool(a), Val::Bool(b)) => Val::Bool(a | b),
                        _ => self.fatal("invalid 'and' operation"),
                    };
                    self.push(result);
                }
                opcode::xor => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Int(a ^ b),
                        (Val::Bool(a), Val::Bool(b)) => Val::Bool(a ^ b),
                        _ => self.fatal("invalid 'and' operation"),
                    };
                    self.push(result);
                }
                opcode::exp => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(a) => Val::Float(a.exp()),
                        _ => self.fatal("invalid 'exp' operation"),
                    };
                    self.push(result);
                }
                opcode::ln => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(a) => Val::Float(a.ln()),
                        _ => self.fatal("invalid 'ln' operation"),
                    };
                    self.push(result);
                }
                opcode::pos => {
                    let val = self.pop();
                    let result = match val {
                        Val::Int(a) => Val::Int(a),
                        Val::Float(a) => Val::Float(a),
                        Val::Bool(a) => Val::Bool(a),
                        _ => self.fatal("invalid 'pos' operation"),
                    };
                    self.push(result);
                }
                opcode::neg => {
                    let val = self.pop();
                    let result = match val {
                        Val::Int(a) => Val::Int(-a),
                        Val::Float(a) => Val::Float(-a),
                        _ => self.fatal("invalid 'neg' operation"),
                    };
                    self.push(result);
                }
                opcode::not => {
                    let val = self.pop();
                    let result = match val {
                        Val::Int(a) => Val::Int(!a),
                        Val::Bool(a) => Val::Bool(!a),
                        _ => self.fatal("invalid 'not' operation"),
                    };
                    self.push(result);
                }
                opcode::eq => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a == b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a == b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) == self.heap.deref_string(b))
                        }
                        (Val::Bool(a), Val::Bool(b)) => Val::Bool(a == b),
                        _ => self.fatal("invalid 'eq' operation"),
                    };
                    self.push(result);
                }
                opcode::ne => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a != b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a != b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) != self.heap.deref_string(b))
                        }
                        (Val::Bool(a), Val::Bool(b)) => Val::Bool(a != b),
                        _ => self.fatal("invalid 'ne' operation"),
                    };
                    self.push(result);
                }
                opcode::lt => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a < b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a < b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) < self.heap.deref_string(b))
                        }
                        _ => self.fatal("invalid 'lt' operation"),
                    };
                    self.push(result);
                }
                opcode::le => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a <= b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a <= b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) <= self.heap.deref_string(b))
                        }
                        _ => self.fatal("invalid 'le' operation"),
                    };
                    self.push(result);
                }
                opcode::gt => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a > b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a > b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) > self.heap.deref_string(b))
                        }
                        _ => self.fatal("invalid 'gt' operation"),
                    };
                    self.push(result);
                }
                opcode::ge => {
                    let right = self.pop();
                    let left = self.pop();
                    let result = match (left, right) {
                        (Val::Int(a), Val::Int(b)) => Val::Bool(a >= b),
                        (Val::Float(a), Val::Float(b)) => Val::Bool(a >= b),
                        (Val::String(a), Val::String(b)) => {
                            Val::Bool(self.heap.deref_string(a) >= self.heap.deref_string(b))
                        }
                        _ => self.fatal("invalid 'ge' operation"),
                    };
                    self.push(result);
                }
                opcode::sin => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.sin()),
                        _ => self.fatal("invalid 'sin' operation"),
                    };
                    self.push(result);
                }
                opcode::cos => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.cos()),
                        _ => self.fatal("invalid 'cos' operation"),
                    };
                    self.push(result);
                }
                opcode::tan => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.tan()),
                        _ => self.fatal("invalid 'tan' operation"),
                    };
                    self.push(result);
                }
                opcode::asin => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.asin()),
                        _ => self.fatal("invalid 'asin' operation"),
                    };
                    self.push(result);
                }
                opcode::acos => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.acos()),
                        _ => self.fatal("invalid 'acos' operation"),
                    };
                    self.push(result);
                }
                opcode::atan => {
                    let val = self.pop();
                    let result = match val {
                        Val::Float(x) => Val::Float(x.atan()),
                        _ => self.fatal("invalid 'atan' operation"),
                    };
                    self.push(result);
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
                        self.fatal("found non-boolean value as jump_if condition");
                    };
                    if b {
                        self.cursor = pos;
                    }
                }
                opcode::jump_if_not => {
                    let pos = self.read_u32() as usize;
                    let Val::Bool(b) = self.pop() else {
                        self.fatal("found non-boolean value as jump_if_not condition");
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

                    let fun_bundle = self.pop();
                    let Val::Bundle(u) = fun_bundle else {
                        self.fatal("invalid function object");
                    };

                    let &[Val::Func(addr), Val::Bundle(u_capture)] = self.heap.deref_val_array(u)
                    else {
                        self.fatal("invalid function bundle");
                    };

                    let captured = self.heap.deref_val_array(u_capture);
                    self.stack.extend_from_slice(captured);

                    self.push_call_frame(arg_count + captured.len());
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
                opcode::pop_offset => {
                    let offset = self.read_u16() as usize;
                    let index = self.stack.len() - offset - 1;
                    self.stack.remove(index);
                }
                opcode::dup => {
                    let value = self.peek().clone();
                    self.push(value);
                }
                opcode::panic => {
                    let val = self.pop();
                    let msg = self.to_user_val(&val).to_string();
                    self.fatal(format!("PANIC: {msg}").as_str());
                }
                _ => self.fatal(format!("invalid opcode 0x{op:x}").as_str()),
            }
        }

        let result = self.pop();
        debug_assert!(self.stack.is_empty(), "non-empty stack after halting");
        self.to_user_val(&result)
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

    fn read_u64(&mut self) -> u64 {
        u64::from_le_bytes([
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
        ])
    }
}

type HeapIndex = usize;

#[derive(Debug)]
struct Heap {
    strings: Vec<String>,
    values: Vec<Val>,
    value_stride_lengths: Vec<usize>,
}

impl Heap {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            values: Vec::new(),
            value_stride_lengths: Vec::new(),
        }
    }

    fn alloc_string(&mut self, string: String) -> HeapIndex {
        let index = self.strings.len();
        self.strings.push(string);
        index
    }

    // fn alloc_val(&mut self, val: Val) -> HeapIndex {
    //     let index = self.values.len();
    //     self.values.push(val);
    //     self.value_stride_lengths.push(1);
    //     index
    // }

    fn alloc_val_array(&mut self, mut vals: Vec<Val>) -> HeapIndex {
        let index = self.values.len();
        let len = vals.len();
        if len > 0 {
            self.values.append(&mut vals);
            self.value_stride_lengths.push(len);
            self.value_stride_lengths.append(&mut vec![1; len - 1]);
        } else {
            // todo: make unit values not cause a pointless allocation
            self.values.push(Val::Nil);
            self.value_stride_lengths.push(0);
        }
        index
    }

    fn deref_string(&self, index: HeapIndex) -> &str {
        &self.strings[index]
    }

    fn deref_val(&self, index: HeapIndex, offset: usize) -> &Val {
        &self.values[index + offset]
    }

    fn deref_val_array(&self, index: HeapIndex) -> &[Val] {
        let len = self.value_stride_lengths[index];
        &self.values[index..(index + len)]
    }
}

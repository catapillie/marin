use byteorder::{LE, WriteBytesExt};

use super::{
    ir,
    low::{self, FunID},
};
use crate::{
    binary::{self, Opcode, opcode},
    exe::Value,
};
use std::{collections::HashMap, io::Cursor};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Marker(usize);

#[derive(Default)]
struct MarkerInfo {
    incoming: Vec<Marker>,
    outgoing: Option<(Marker, JumpMode)>,
}

#[derive(Clone)]
enum Placeholder {
    Unpatched(Vec<u32>),
    Patched(u32),
}

struct Label {
    depth: usize,
    breaks: Vec<Marker>,
    skips: Vec<Marker>,
}

#[derive(Debug)]
enum JumpMode {
    Always,
    IfTrue,
    IfFalse,
    Eq,
}

enum PseudoOp {
    Noop,
    Op(Opcode),
    LoadFun(low::FunID),
}

struct BytecodeBuilder {
    constants: Vec<Value>,

    function_table: HashMap<u32, String>,
    function_positions: Vec<Placeholder>,
    labels: HashMap<ir::LabelID, Label>,
    frame_depth: usize,

    opcodes: Vec<(PseudoOp, Option<Marker>)>,
    cursor: Cursor<Vec<u8>>,
    markers: Vec<MarkerInfo>,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self {
            constants: Vec::new(),

            function_table: HashMap::new(),
            function_positions: Vec::new(),
            labels: HashMap::new(),
            frame_depth: 0,

            opcodes: Vec::new(),
            cursor: Cursor::new(Vec::new()),
            markers: Vec::new(),
        }
    }

    fn pos(&self) -> u32 {
        self.cursor.position() as u32
    }

    fn write_opcode(&mut self, opcode: Opcode) {
        self.opcodes.push((PseudoOp::Op(opcode), None))
    }

    fn write_load_fun(&mut self, id: FunID) {
        self.opcodes.push((PseudoOp::LoadFun(id), None))
    }

    // multiple cases for marking the current opcode
    // 1. there is already a marker on the latest opcode
    //     => push a no-op and place a new marker there
    // 2. the latest opcode isn't marked
    //     => place a new marker on it
    // 3. there are no opcodes yet
    //     => return the 'top' marker (0)
    fn mark(&mut self) -> Marker {
        match self.opcodes.last_mut() {
            Some((_, Some(_))) => {
                let m = Marker(self.markers.len());
                self.markers.push(MarkerInfo::default());
                self.opcodes.push((PseudoOp::Noop, Some(m)));
                m
            }
            Some((_, marker @ None)) => {
                let m = Marker(self.markers.len());
                self.markers.push(MarkerInfo::default());
                *marker = Some(m);
                m
            }
            None => Marker(0), // top marker
        }
    }

    fn wire_jump(&mut self, mode: JumpMode, from: Marker, to: Marker) {
        if let Some((old_to, _)) = self.markers[from.0].outgoing {
            panic!(
                "wiring {}->{} overrides {}->{}",
                from.0, to.0, from.0, old_to.0
            )
        }

        self.markers[from.0].outgoing = Some((to, mode));
        self.markers[to.0].incoming.push(from);
    }

    fn build_program(&mut self, program: low::Program) -> binary::Result<()> {
        self.function_positions = vec![Placeholder::Unpatched(vec![]); program.functions.len()];

        for fun in program.functions {
            self.build_function(fun)?;
        }
        Ok(())
    }

    fn build_function(&mut self, function: low::Function) -> binary::Result<()> {
        // reset state
        self.opcodes.clear();
        self.markers.clear();
        self.markers.push(MarkerInfo::default()); // top marker

        // patch placeholders to this function, and register function position
        let fun_pos = self.pos();
        if let Placeholder::Unpatched(unpatched) = &self.function_positions[function.id.0] {
            for pos in unpatched {
                self.cursor.set_position(*pos as u64);
                self.cursor.write_u32::<LE>(fun_pos)?;
            }
            self.cursor.set_position(fun_pos as u64);
        }
        self.function_positions[function.id.0] = Placeholder::Patched(fun_pos);

        // deconstruct arguments, make sure to skip over captured locals
        let size = function.args.len() as u16;
        for (i, item) in function.args.into_iter().enumerate() {
            let offset = size - 1 - i as u16 + function.captured_count as u16;
            self.build_deconstruct(item, offset);
        }

        // generate expression and return
        self.build_expression(function.expr);
        self.write_opcode(Opcode::ret);

        self.function_table.insert(fun_pos, function.name);
        self.emit_bytecode()?;
        Ok(())
    }

    fn build_do_frame(&mut self) {
        self.write_opcode(Opcode::do_frame);
        self.frame_depth += 1;
    }

    fn build_end_frame(&mut self) {
        self.write_opcode(Opcode::end_frame);
        self.frame_depth -= 1;
    }

    fn register_label(&mut self, label: ir::LabelID) {
        self.labels.insert(
            label,
            Label {
                depth: self.frame_depth,
                breaks: Vec::new(),
                skips: Vec::new(),
            },
        );
    }

    fn get_label(&self, label_id: ir::LabelID) -> &Label {
        let Some(label) = self.labels.get(&label_id) else {
            panic!("unregistered label '{:?}'", label_id)
        };
        label
    }

    fn get_label_mut(&mut self, label_id: ir::LabelID) -> &mut Label {
        let Some(label) = self.labels.get_mut(&label_id) else {
            panic!("unregistered label '{:?}'", label_id)
        };
        label
    }

    fn wire_label_breaks(&mut self, label_id: ir::LabelID, dest: Marker) {
        let breaks = std::mem::take(&mut self.get_label_mut(label_id).breaks);
        for marker in breaks {
            self.wire_jump(JumpMode::Always, marker, dest);
        }
    }

    fn wire_label_skips(&mut self, label_id: ir::LabelID, dest: Marker) {
        let skips = std::mem::take(&mut self.get_label_mut(label_id).skips);
        for marker in skips {
            self.wire_jump(JumpMode::Always, marker, dest);
        }
    }

    fn build_statement(&mut self, stmt: low::Stmt) {
        use low::Stmt as S;
        match stmt {
            S::Nothing => {}
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
            S::Block { stmts, needs_frame } => {
                if needs_frame {
                    self.write_opcode(Opcode::do_frame);
                }
                for stmt in stmts {
                    self.build_statement(stmt);
                }
                if needs_frame {
                    self.write_opcode(Opcode::end_frame);
                }
            }
        }
    }

    fn build_deconstruct(&mut self, pat: low::Pat, at: u16) {
        use low::Pat as P;
        match pat {
            P::Discard | P::Int(_) | P::Float(_) | P::String(_) | P::Bool(_) => {
                self.write_opcode(Opcode::pop_offset(at));
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

    fn build_unwrapping(&mut self, mut unwrapping: low::Unwrapping) {
        use low::Unwrapping as U;
        while let U::Bundle { index, next } = unwrapping {
            let index: u8 = index.try_into().expect("cannot index beyond 255");
            self.write_opcode(Opcode::index(index));
            unwrapping = *next;
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
            E::Access { accessed, index } => {
                self.build_expression(*accessed);
                self.write_opcode(Opcode::index(index));
            }
            E::Block {
                label,
                stmts,
                result,
                needs_frame,
            } => self.build_block_expression(label, stmts, *result, needs_frame),
            E::Variant { tag, items } => {
                self.build_constant(Value::Int(tag));
                self.build_small_bundle(items);
                self.write_opcode(Opcode::bundle(2));
            }
            E::Local { local } => self.build_local(local),
            E::If {
                label,
                guard,
                then_branch,
                else_branch,
            } => self.build_if(*guard, *then_branch, *else_branch, label),
            E::While {
                label,
                guard,
                do_branch,
                else_branch,
            } => self.build_while(*guard, *do_branch, *else_branch, label),
            E::Loop { label, body } => self.build_loop(*body, label),
            E::Fun { id, captured } => self.build_fun(id, captured),
            E::Call { callee, args } => {
                let arg_count: u8 = args
                    .len()
                    .try_into()
                    .expect("function call cannot have more than 255 arguments");
                for arg in args {
                    self.build_expression(arg);
                }
                self.build_expression(*callee);
                self.write_opcode(Opcode::call(arg_count));
            }
            E::Unwrap { value, unwrapping } => {
                self.build_expression(*value);
                self.build_unwrapping(unwrapping);
            }
            E::Match {
                scrutinee,
                decision,
                fallback,
            } => self.build_match(*scrutinee, *decision, *fallback),
            E::Break { value, label } => self.build_break(*value, label),
            E::Skip { label } => self.build_skip(label),

            E::Add(left, right) => self.build_binary_op(*left, *right, Opcode::add),
            E::Sub(left, right) => self.build_binary_op(*left, *right, Opcode::sub),
            E::Mul(left, right) => self.build_binary_op(*left, *right, Opcode::mul),
            E::Div(left, right) => self.build_binary_op(*left, *right, Opcode::div),
            E::Mod(left, right) => self.build_binary_op(*left, *right, Opcode::modulo),
            E::BitAnd(left, right) => self.build_binary_op(*left, *right, Opcode::and),
            E::BitOr(left, right) => self.build_binary_op(*left, *right, Opcode::or),
            E::BitXor(left, right) => self.build_binary_op(*left, *right, Opcode::xor),

            E::ShortAnd(left, right) => self.build_short_circuit_and(*left, *right),
            E::ShortOr(left, right) => self.build_short_circuit_or(*left, *right),

            E::Pos(arg) => self.build_unary_op(*arg, Opcode::pos),
            E::Neg(arg) => self.build_unary_op(*arg, Opcode::neg),
            E::BitNeg(arg) => self.build_unary_op(*arg, Opcode::not),

            E::Pow(left, right) => self.build_binary_op(*left, *right, Opcode::pow),
            E::Exp(arg) => self.build_unary_op(*arg, Opcode::exp),
            E::Ln(arg) => self.build_unary_op(*arg, Opcode::ln),
            E::Sin(arg) => self.build_unary_op(*arg, Opcode::sin),
            E::Cos(arg) => self.build_unary_op(*arg, Opcode::cos),
            E::Tan(arg) => self.build_unary_op(*arg, Opcode::tan),
            E::Asin(arg) => self.build_unary_op(*arg, Opcode::asin),
            E::Acos(arg) => self.build_unary_op(*arg, Opcode::acos),
            E::Atan(arg) => self.build_unary_op(*arg, Opcode::atan),

            E::Eq(left, right) => self.build_binary_op(*left, *right, Opcode::eq),
            E::Ne(left, right) => self.build_binary_op(*left, *right, Opcode::ne),
            E::Lt(left, right) => self.build_binary_op(*left, *right, Opcode::lt),
            E::Le(left, right) => self.build_binary_op(*left, *right, Opcode::le),
            E::Gt(left, right) => self.build_binary_op(*left, *right, Opcode::gt),
            E::Ge(left, right) => self.build_binary_op(*left, *right, Opcode::ge),

            E::Panic(arg) => self.build_unary_op(*arg, Opcode::panic),
        }
    }

    fn build_constant(&mut self, value: Value) {
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

    fn build_local(&mut self, local: u8) {
        self.write_opcode(Opcode::load_local(local))
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

    fn build_block_expression(
        &mut self,
        label: Option<ir::LabelID>,
        stmts: Box<[low::Stmt]>,
        result: low::Expr,
        needs_frame: bool,
    ) {
        if let Some(label) = label {
            self.register_label(label);
        }
        if needs_frame {
            self.build_do_frame();
        }

        for stmt in stmts {
            self.build_statement(stmt);
        }
        self.build_expression(result);

        if needs_frame {
            self.build_end_frame();
        }
        let block_end = self.mark();
        if let Some(label) = label {
            self.wire_label_breaks(label, block_end);
        }
    }

    fn build_if(
        &mut self,
        guard: low::Expr,
        then_branch: low::Expr,
        else_branch: low::Expr,
        label: ir::LabelID,
    ) {
        self.register_label(label);

        self.build_expression(guard);
        let guard_end_marker = self.mark();

        self.build_expression(then_branch);
        let then_end_marker = self.mark();

        self.build_expression(else_branch);
        let else_end_marker = self.mark();

        self.wire_jump(JumpMode::IfFalse, guard_end_marker, then_end_marker);
        self.wire_jump(JumpMode::Always, then_end_marker, else_end_marker);
        self.wire_label_breaks(label, else_end_marker);
    }

    fn build_while(
        &mut self,
        guard: low::Expr,
        do_branch: low::Stmt,
        else_branch: low::Expr,
        label: ir::LabelID,
    ) {
        self.register_label(label);

        let guard_start_marker = self.mark();
        self.build_expression(guard);
        let guard_end_marker = self.mark();

        self.build_statement(do_branch);
        let block_end_marker = self.mark();

        self.build_expression(else_branch);
        let else_end_marker = self.mark();

        self.wire_jump(JumpMode::IfFalse, guard_end_marker, block_end_marker);
        self.wire_jump(JumpMode::Always, block_end_marker, guard_start_marker);
        self.wire_label_breaks(label, else_end_marker);
        self.wire_label_skips(label, guard_start_marker);
    }

    fn build_loop(&mut self, body: low::Stmt, label: ir::LabelID) {
        self.register_label(label);

        let loop_start_marker = self.mark();
        self.build_statement(body);
        let loop_end_marker = self.mark();

        self.wire_jump(JumpMode::Always, loop_end_marker, loop_start_marker);
        self.wire_label_breaks(label, loop_end_marker);
        self.wire_label_skips(label, loop_start_marker);
    }

    fn build_match(&mut self, scrutinee: low::Expr, decision: low::Decision, fallback: low::Expr) {
        self.write_opcode(Opcode::do_frame);

        self.build_expression(scrutinee);

        let mut failure_markers = Vec::new();
        let mut success_markers = Vec::new();
        self.build_decision(decision, &mut success_markers, &mut failure_markers);

        let failure_dest_marker = self.mark();
        for marker in failure_markers {
            self.wire_jump(JumpMode::Always, marker, failure_dest_marker);
        }

        self.build_expression(fallback);

        let success_dest_marker = self.mark();
        for marker in success_markers {
            self.wire_jump(JumpMode::Always, marker, success_dest_marker);
        }

        self.write_opcode(Opcode::end_frame);
    }

    fn build_decision(
        &mut self,
        decision: low::Decision,
        success_markers: &mut Vec<Marker>,
        failure_markers: &mut Vec<Marker>,
    ) {
        use low::Decision as D;
        use low::Pat as P;
        match decision {
            D::Failure => {
                let marker = self.mark();
                failure_markers.push(marker);
            }
            D::Success { expr } => {
                self.build_expression(*expr);
                let marker = self.mark();
                success_markers.push(marker);
            }
            D::Test {
                local,
                pat,
                success,
                failure,
            } => {
                self.build_local(local);
                let (test_jump_mode, deconstruct_pattern) = match *pat {
                    P::Discard => {
                        self.write_opcode(Opcode::pop);
                        (JumpMode::Always, None)
                    }
                    P::Int(val) => {
                        self.build_constant(Value::Int(val));
                        (JumpMode::Eq, None)
                    }
                    P::Float(val) => {
                        self.build_constant(Value::Float(val));
                        (JumpMode::Eq, None)
                    }
                    P::String(val) => {
                        self.build_constant(Value::String(val));
                        (JumpMode::Eq, None)
                    }
                    P::Bool(val) => {
                        self.build_constant(Value::Bool(val));
                        (JumpMode::Eq, None)
                    }
                    P::Local(loc) => {
                        self.write_opcode(Opcode::pop);
                        (JumpMode::Always, Some(P::Local(loc)))
                    }
                    P::Bundle(pats) => {
                        self.write_opcode(Opcode::pop);
                        (JumpMode::Always, Some(P::Bundle(pats)))
                    }
                    P::Variant(tag, pats) => {
                        self.write_opcode(Opcode::index(0));
                        self.build_constant(Value::Int(tag as i64));
                        (JumpMode::Eq, Some(P::Variant(tag, pats)))
                    }
                };

                let failure_begin_marker = self.mark();
                self.build_decision(*failure, success_markers, failure_markers);

                let success_begin_marker = self.mark();
                if let Some(pat) = deconstruct_pattern {
                    self.build_local(local);
                    self.build_deconstruct(pat, 0);
                }
                self.build_decision(*success, success_markers, failure_markers);

                self.wire_jump(test_jump_mode, failure_begin_marker, success_begin_marker);
            }
        }
    }

    fn build_fun(&mut self, id: low::FunID, captured: Box<[u8]>) {
        self.write_load_fun(id);

        // captured item bundle
        let captured_local_count: u8 = captured
            .len()
            .try_into()
            .expect("function cannot capture more than 255 items");
        for local in captured {
            self.write_opcode(Opcode::load_local(local));
        }
        self.write_opcode(Opcode::bundle(captured_local_count));

        // (fun, [captured...])
        self.write_opcode(Opcode::bundle(2));
    }

    fn emit_bytecode(&mut self) -> binary::Result<()> {
        // initialize marker gen info
        let mut placeholders = vec![Placeholder::Unpatched(vec![]); self.markers.len()];
        placeholders[0] = Placeholder::Patched(0); // top marker

        for (opcode, marker) in &self.opcodes {
            match opcode {
                // no operation
                PseudoOp::Noop => {}

                // simple opcode
                PseudoOp::Op(op) => binary::write_opcode(&mut self.cursor, op)?,

                // 'load_fun' opcode might have to use a placeholder
                PseudoOp::LoadFun(id) => {
                    self.cursor.write_u8(opcode::load_fun)?;
                    let pos = self.pos();
                    match &mut self.function_positions[id.0] {
                        Placeholder::Unpatched(unpatched) => {
                            unpatched.push(pos);
                            self.cursor.write_u32::<LE>(0)?;
                        }
                        Placeholder::Patched(pos) => self.cursor.write_u32::<LE>(*pos)?,
                    }
                }
            }

            if let Some(marker) = marker {
                // emit jumps from here to other markers
                if let Some((dest, mode)) = &self.markers[marker.0].outgoing {
                    match mode {
                        JumpMode::Always => self.cursor.write_u8(opcode::jump)?,
                        JumpMode::IfTrue => self.cursor.write_u8(opcode::jump_if)?,
                        JumpMode::IfFalse => self.cursor.write_u8(opcode::jump_if_not)?,
                        JumpMode::Eq => self.cursor.write_u8(opcode::jump_eq)?,
                    }

                    match &mut placeholders[dest.0] {
                        Placeholder::Unpatched(unpatched) => {
                            unpatched.push(self.pos());
                            self.cursor.write_u32::<LE>(0)?;
                        }
                        Placeholder::Patched(dest_pos) => self.cursor.write_u32::<LE>(*dest_pos)?,
                    }
                }

                // patch or emit jumps to this marker
                let marker_pos = self.pos();
                if let Placeholder::Unpatched(unpatched) = &placeholders[marker.0] {
                    for pos in unpatched {
                        self.cursor.set_position(*pos as u64);
                        self.cursor.write_u32::<LE>(marker_pos)?;
                    }
                    self.cursor.set_position(marker_pos as u64);
                }
                placeholders[marker.0] = Placeholder::Patched(marker_pos);
            }
        }
        Ok(())
    }

    fn build_binary_op(&mut self, left: low::Expr, right: low::Expr, op: Opcode) {
        self.build_expression(left);
        self.build_expression(right);
        self.write_opcode(op);
    }

    fn build_unary_op(&mut self, arg: low::Expr, op: Opcode) {
        self.build_expression(arg);
        self.write_opcode(op);
    }

    fn build_short_circuit_and(&mut self, left: low::Expr, right: low::Expr) {
        self.build_expression(left);
        self.write_opcode(Opcode::dup);

        let right_start_marker = self.mark();
        self.write_opcode(Opcode::pop);
        self.build_expression(right);
        let right_end_marker = self.mark();

        self.wire_jump(JumpMode::IfFalse, right_start_marker, right_end_marker);
    }

    fn build_short_circuit_or(&mut self, left: low::Expr, right: low::Expr) {
        self.build_expression(left);
        self.write_opcode(Opcode::dup);

        let right_start_marker = self.mark();
        self.write_opcode(Opcode::pop);
        self.build_expression(right);
        let right_end_marker = self.mark();

        self.wire_jump(JumpMode::IfTrue, right_start_marker, right_end_marker);
    }

    fn build_break(&mut self, value: low::Expr, label: ir::LabelID) {
        self.build_expression(value);

        let current_depth = self.frame_depth;
        let label_depth = self.get_label(label).depth;
        debug_assert!(current_depth >= label_depth);

        let frame_distance = current_depth - label_depth;
        for _ in 0..frame_distance {
            self.write_opcode(Opcode::end_frame);
        }
        let break_marker = self.mark();
        self.get_label_mut(label).breaks.push(break_marker);
    }

    fn build_skip(&mut self, label: ir::LabelID) {
        let current_depth = self.frame_depth;
        let label_depth = self.get_label(label).depth;
        debug_assert!(current_depth >= label_depth);

        let frame_distance = current_depth - label_depth;
        for _ in 0..frame_distance {
            self.write_opcode(Opcode::end_frame);
        }
        let skip_marker = self.mark();
        self.get_label_mut(label).skips.push(skip_marker);
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

use byteorder::{LE, WriteBytesExt};

use super::low::{self, FunID};
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

#[derive(Debug)]
enum JumpMode {
    Always,
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
    function_captures: Vec<Vec<u8>>,

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
            function_captures: Vec::new(),

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

    fn build_program(&mut self, mut program: low::Program) -> binary::Result<()> {
        self.function_positions = vec![Placeholder::Unpatched(vec![]); program.functions.len()];
        self.function_captures = vec![vec![]; program.functions.len()];

        for function in &mut program.functions {
            self.function_captures[function.id.0] = std::mem::take(&mut function.captured_locals);
        }

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

        // deconstruct arguments
        let size = function.args.len() as u16;
        for (i, item) in function.args.into_iter().enumerate() {
            let offset = size - 1 - i as u16;
            self.build_deconstruct(item, offset);
        }

        // generate expression and return
        self.build_expression(function.expr);
        self.write_opcode(Opcode::ret);

        self.function_table.insert(fun_pos, function.name);
        self.emit_bytecode()?;
        Ok(())
    }

    fn build_statement(&mut self, stmt: low::Stmt) {
        use low::Stmt as S;
        match stmt {
            S::Nothing => {},
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
            E::Block {
                stmts,
                result,
                needs_frame,
            } => {
                if needs_frame {
                    self.write_opcode(Opcode::do_frame);
                }

                for stmt in stmts {
                    self.build_statement(stmt);
                }
                self.build_expression(*result);

                if needs_frame {
                    self.write_opcode(Opcode::end_frame);
                }
            }
            E::Variant { tag, items } => {
                self.build_constant(Value::Int(tag));
                self.build_small_bundle(items);
                self.write_opcode(Opcode::bundle(2));
            }
            E::Local { local } => self.build_local(local),
            E::If {
                guard,
                then_branch,
                else_branch,
            } => self.build_if(*guard, *then_branch, *else_branch),
            E::While {
                guard,
                do_branch,
                else_branch,
            } => self.build_while(*guard, *do_branch, *else_branch),
            E::Loop { body } => self.build_loop(*body),
            E::Fun { id } => self.build_fun(id),
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

    fn build_if(&mut self, guard: low::Expr, then_branch: low::Expr, else_branch: low::Expr) {
        self.build_expression(guard);
        let guard_end_marker = self.mark();

        self.build_expression(then_branch);
        let then_end_marker = self.mark();

        self.build_expression(else_branch);
        let else_end_marker = self.mark();

        self.wire_jump(JumpMode::IfFalse, guard_end_marker, then_end_marker);
        self.wire_jump(JumpMode::Always, then_end_marker, else_end_marker);
    }

    fn build_while(&mut self, guard: low::Expr, do_branch: low::Stmt, else_branch: low::Expr) {
        let guard_start_marker = self.mark();
        self.build_expression(guard);
        let guard_end_marker = self.mark();

        self.build_statement(do_branch);
        let block_end_marker = self.mark();

        self.build_expression(else_branch);

        self.wire_jump(JumpMode::IfFalse, guard_end_marker, block_end_marker);
        self.wire_jump(JumpMode::Always, block_end_marker, guard_start_marker);
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

    fn build_loop(&mut self, body: low::Stmt) {
        let loop_start_marker = self.mark();
        self.build_statement(body);
        let loop_end_marker = self.mark();

        self.wire_jump(JumpMode::Always, loop_end_marker, loop_start_marker);
    }

    fn build_fun(&mut self, id: low::FunID) {
        self.write_load_fun(id);

        // captured item bundle
        let captured_locals = self.function_captures[id.0].clone();
        let captured_local_count: u8 = captured_locals
            .len()
            .try_into()
            .expect("function cannot capture more than 255 items");
        for local in captured_locals {
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

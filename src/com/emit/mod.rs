use byteorder::{LE, WriteBytesExt};

use super::low;
use crate::{
    binary::{self, Opcode, opcode},
    exe::Value,
};
use std::{collections::HashMap, io::Cursor};

#[derive(Clone, Copy)]
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

enum JumpMode {
    Always,
    IfTrue,
    IfFalse,
}

enum PseudoOp {
    Op(Opcode),
}

struct BytecodeBuilder {
    constants: Vec<Value>,
    function_table: HashMap<u32, String>,
    opcodes: Vec<(PseudoOp, Option<Marker>)>,
    cursor: Cursor<Vec<u8>>,
    markers: Vec<MarkerInfo>,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            function_table: HashMap::new(),
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

    fn mark(&mut self) -> Marker {
        match self.opcodes.last_mut() {
            Some((_, Some(marker))) => *marker,
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
        self.markers[from.0].outgoing = Some((to, mode));
        self.markers[to.0].incoming.push(from);
    }

    fn build_program(&mut self, program: low::Program) -> binary::Result<()> {
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
            E::Local { local } => {
                self.write_opcode(Opcode::load_local(local));
            }
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

    fn build_loop(&mut self, body: low::Stmt) {
        let loop_start_marker = self.mark();
        self.build_statement(body);
        let loop_end_marker = self.mark();

        self.wire_jump(JumpMode::Always, loop_end_marker, loop_start_marker);
    }

    fn emit_bytecode(&mut self) -> binary::Result<()> {
        // initialize marker gen info
        let mut placeholders = vec![Placeholder::Unpatched(vec![]); self.markers.len()];
        placeholders[0] = Placeholder::Patched(0); // top marker

        for (opcode, marker) in &self.opcodes {
            match opcode {
                PseudoOp::Op(op) => binary::write_opcode(&mut self.cursor, op)?,
            }

            if let Some(marker) = marker {
                // emit jumps from here to other markers
                if let Some((dest, mode)) = &self.markers[marker.0].outgoing {
                    match mode {
                        JumpMode::Always => self.cursor.write_u8(opcode::jump)?,
                        JumpMode::IfTrue => self.cursor.write_u8(opcode::jump_if)?,
                        JumpMode::IfFalse => self.cursor.write_u8(opcode::jump_if_not)?,
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

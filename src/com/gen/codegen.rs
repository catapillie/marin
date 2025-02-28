use std::io::Cursor;

use crate::{
    binary::{self, Opcode},
    com::ir,
    exe,
};

pub struct Codegen<'a> {
    ir: &'a [ir::Module],
    constants: Vec<exe::Value>,
    cursor: Cursor<Vec<u8>>,
}

impl<'a> Codegen<'a> {
    pub fn new(ir: &'a [ir::Module]) -> Self {
        Self {
            ir,
            constants: Vec::new(),
            cursor: Cursor::new(Vec::new()),
        }
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

    pub fn gen(&mut self) -> binary::Result<()> {
        for module in self.ir {
            self.gen_module(module)?;
        }
        Ok(())
    }

    fn gen_module(&mut self, module: &'a ir::Module) -> binary::Result<()> {
        for stmt in &module.stmts {
            self.gen_statement(stmt)?;
        }
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
            S::Let(_, _) => todo!("let statement"),
        }
    }

    fn gen_expression(&mut self, expr: &'a ir::Expr) -> binary::Result<()> {
        use ir::Expr as E;
        match expr {
            E::Missing => Ok(()),
            E::Int(n) => {
                let id = self.add_constant(exe::Value::Int(*n));
                self.write_opcode(&Opcode::ld_const(id))?;
                Ok(())
            }
            E::Float(f) => {
                let id = self.add_constant(exe::Value::Float(*f));
                self.write_opcode(&Opcode::ld_const(id))?;
                Ok(())
            }
            E::String(s) => {
                let id = self.add_constant(exe::Value::String(s.clone()));
                self.write_opcode(&Opcode::ld_const(id))?;
                Ok(())
            }
            E::Bool(b) => {
                let id = self.add_constant(exe::Value::Bool(*b));
                self.write_opcode(&Opcode::ld_const(id))?;
                Ok(())
            }
            E::Var(..) => todo!(),
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
            E::Block(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Fun(..) => todo!(),
            E::Call(..) => todo!(),
            E::Variant(tag, items) => {
                // gen tag as i64
                let tag_id = self.add_constant(exe::Value::Int(*tag as i64));
                self.write_opcode(&Opcode::ld_const(tag_id))?;

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

    pub fn done(self) -> binary::Result<Vec<u8>> {
        let mut code = self.cursor.into_inner();

        let final_size_approx = code.len() + binary::MAGIC.len() + 1;
        let mut bytecode = Vec::with_capacity(final_size_approx);

        binary::write_magic(&mut bytecode)?;
        binary::write_constant_pool(&mut bytecode, &self.constants)?;

        bytecode.append(&mut code);
        binary::write_opcode(&mut bytecode, &Opcode::halt)?;

        Ok(bytecode)
    }
}

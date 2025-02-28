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
                Ok(())
            }
            S::Let(_, _) => todo!("let statement"),
        }
    }

    fn gen_expression(&mut self, expr: &'a ir::Expr) -> binary::Result<()> {
        use ir::Expr as E;
        match expr {
            E::Missing => Ok(()),
            E::Int(..) => todo!(),
            E::Float(..) => todo!(),
            E::String(..) => todo!(),
            E::Bool(..) => todo!(),
            E::Var(..) => todo!(),
            E::Tuple(..) => todo!(),
            E::Array(..) => todo!(),
            E::Block(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
            E::Skip(..) => todo!(),
            E::Fun(..) => todo!(),
            E::Call(..) => todo!(),
            E::Variant(..) => todo!(),
            E::Record(..) => todo!(),
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

        Ok(bytecode)
    }
}

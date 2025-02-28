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
        binary::write_opcode(&mut self.cursor, &Opcode::halt)?;
        Ok(())
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

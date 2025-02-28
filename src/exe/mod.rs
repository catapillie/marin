pub mod vm;
pub use vm::VM;

pub mod value;
pub use value::Value;

use crate::binary;

pub fn run_bytecode(mut bytecode: &[u8]) {
    binary::read_magic(&mut bytecode).expect("not marin bytecode (magic bytes mismatch)");
    let constants =
        binary::read_constant_pool(&mut bytecode).expect("not marin bytecode (invalid format)");

    let mut vm = VM::new(bytecode);
    for value in constants {
        vm.add_constant(&value);
    }
    vm.run();
}

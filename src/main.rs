use exe::{opcode, Value};

mod com;
mod exe;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let mut compiler = com::init();

    compiler.add_marin_std();
    for arg in &args {
        compiler.add_file(arg);
    }

    let compiler = compiler.read_sources().parse().check();

    use codespan_reporting::term::{self, termcolor::ColorChoice};
    let color = ColorChoice::AlwaysAnsi;
    let config = term::Config::default();
    compiler
        .emit_reports(color, &config)
        .expect("failed to emit reports");
    if compiler.is_fatal() {
        std::process::exit(1);
    }

    compiler.gen();

    let mut bytecode = Vec::new();
    {
        bytecode.push(opcode::LD_CONST);
        bytecode.extend_from_slice(&0_u16.to_le_bytes());
        bytecode.push(opcode::LD_CONST);
        bytecode.extend_from_slice(&1_u16.to_le_bytes());
        bytecode.push(opcode::LD_CONST);
        bytecode.extend_from_slice(&2_u16.to_le_bytes());

        bytecode.extend_from_slice(&[opcode::BUNDLE, 3]);
        bytecode.push(opcode::POP);

        bytecode.push(opcode::HALT);
    }

    let mut vm = exe::VM::new(&bytecode);
    vm.add_constant(&Value::Int(42));
    vm.add_constant(&Value::Float(5.42));
    vm.add_constant(&Value::Bool(true));

    vm.run();
}

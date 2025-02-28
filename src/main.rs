mod binary;
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
        binary::write_magic(&mut bytecode).unwrap();
        binary::write_constant_pool(
            &mut bytecode,
            &[
                exe::Value::Int(42),
                exe::Value::Float(4.57),
                exe::Value::Bool(true),
            ],
        )
        .unwrap();

        binary::write_opcode(&mut bytecode, &binary::Opcode::ld_const(0)).unwrap();
        binary::write_opcode(&mut bytecode, &binary::Opcode::ld_const(1)).unwrap();
        binary::write_opcode(&mut bytecode, &binary::Opcode::ld_const(2)).unwrap();

        binary::write_opcode(&mut bytecode, &binary::Opcode::bundle(3)).unwrap();
        binary::write_opcode(&mut bytecode, &binary::Opcode::pop).unwrap();

        binary::write_opcode(&mut bytecode, &binary::Opcode::halt).unwrap();
    }

    let mut cursor = std::io::Cursor::new(&bytecode);
    binary::dissasemble(&mut cursor).unwrap();

    exe::run_bytecode(&bytecode);
}

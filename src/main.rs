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

    let compiler = compiler.gen();
    let bytecode = compiler.into_content().bytecode;

    let mut cursor = std::io::Cursor::new(&bytecode);
    binary::dissasemble(&mut cursor).unwrap();

    exe::run_bytecode(&bytecode);
}

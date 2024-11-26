use codespan_reporting::term::{self, termcolor::ColorChoice};

mod com;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let color = ColorChoice::AlwaysAnsi;
    let config = term::Config::default();

    let mut compiler = com::init();
    for arg in &args {
        compiler.add_files(arg);
    }

    let compiler = compiler.read_sources().parse();
    compiler
        .emit_reports(color, &config)
        .expect("failed to emit reports");
}

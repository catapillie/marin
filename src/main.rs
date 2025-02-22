mod com;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let mut compiler = com::init();
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

    let contents = compiler.file_contents();
    if contents.is_empty() {
        return;
    } else if contents.len() > 1 {
        eprintln!("\nTODO: evaluation with multiple files");
        return;
    }

    let file_ir = &contents[0].0;
    let mut walker = com::Walker::new();
    if let Err(e) = walker.eval_file(file_ir) {
        eprintln!("error: {e:?}")
    }
}

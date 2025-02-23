use colored::Colorize;

mod com;

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

    let contents = compiler.file_contents();
    let info = compiler.info();

    eprintln!("\n{}", "=== EVALUATION ===".bright_white().on_black());

    let mut walker = com::Walker::new();
    for file_id in &info.evaluation_order {
        let file_ir = &contents[*file_id].0;
        if let Err(e) = walker.eval_file(file_ir) {
            eprintln!("error: {e:?}")
        }
    }
}

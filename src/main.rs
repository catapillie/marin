use colored::Colorize;

mod binary;
mod com;
mod exe;

#[cfg(test)]
mod lang_tests;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let mut compiler = com::init();

    let mut has_std = true;
    let mut show_disassembly = false;
    for arg in &args {
        match arg.as_str() {
            // options
            "--no-std" => has_std = false,
            "--show-disassembly" => show_disassembly = true,
            opt if opt.starts_with("--") => {
                panic!("unknown option '{opt}'");
            }

            // path
            arg => compiler.add_file(arg),
        }
    }

    if has_std {
        compiler.add_marin_std();
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

    let compiler = compiler.emit();
    let bytecode = compiler.into_content().bytecode;

    if show_disassembly {
        let mut cursor = std::io::Cursor::new(&bytecode);
        binary::dissasemble(&mut cursor).unwrap();
    }

    println!();

    let value = exe::run_bytecode(&bytecode);
    println!("-> {}", value.to_string().green());
}

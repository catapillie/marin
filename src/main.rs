mod com;

use com::Compiler;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let mut compiler = Compiler::new();
    for arg in &args {
        compiler.add_files(arg);
    }

    compiler.compile();
}

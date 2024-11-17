use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use com::Parser;

mod com;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let path = args.first().expect("no file path provided");
    let source = std::fs::read_to_string(path).expect("couldn't read file");

    let mut reports = Vec::new();

    let mut parser = Parser::new(&source, &mut reports);
    parser.parse_program();

    let mut files = SimpleFiles::new();
    let file = files.add(path, &source);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for report in reports {
        let diagnostic = report.to_diagnostic(file);
        term::emit(&mut writer.lock(), &config, &files, &diagnostic)
            .expect("failed to write report");
    }
}

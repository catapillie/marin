use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use std::process;

mod com;
use com::{Checker, Parser};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let path = args.first().expect("no file path provided");
    let source = std::fs::read_to_string(path).expect("couldn't read file");

    let mut reports = Vec::new();
    let mut parser = Parser::new(&source, &mut reports);
    let expr = parser.parse_program();

    let mut files = SimpleFiles::new();
    let file = files.add(path, &source);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    if !reports.is_empty() {
        for report in reports {
            let diagnostic = report.to_diagnostic(file);
            term::emit(&mut writer.lock(), &config, &files, &diagnostic)
                .expect("failed to write report");
        }
        process::exit(1);
    }

    let mut reports = Vec::new();
    let mut checker = Checker::new(&source, &mut reports);
    checker.check_module(&expr);
}

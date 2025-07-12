use crate::com::loc::Span;

use super::{com, exe};
use std::path::Path;

// ----------------- LANGUAGE TESTS -----------------

full_test!(empty_program_returns_unit => unit());
full_test!(empty_program_returns_unit_explicitly => unit());

full_test!(returns_true => bool(true));
full_test!(returns_false => bool(false));
full_test!(returns_integer => int(42));
full_test!(returns_float => float(4.87));
full_test!(returns_string => str("hello, world"));
full_test!(returns_func => func([]));

full_test!(returns_tuple_1 => bun([int(8), float(0.5), str("hi")]));
full_test!(returns_tuple_2 => bun([bun([bool(true), float(2.0)]), bun([int(3), float(4.0), int(5)]), str("6")]));

full_test!(returns_array_1 => bun([int(1), int(2), int(3), int(4)]));
full_test!(returns_array_2 => bun([bun([int(1), int(2)]), bun([]), bun([int(3)]), bun([int(4), int(5), int(6)])]));

full_test!(block_empty_is_unit => unit());
full_test!(block_singleton_1 => unit());
full_test!(block_singleton_2 => int(73));
full_test!(block_singleton_3 => bun([str("single"), bun([bool(true), str("block")])]));
full_test!(block_last_is_result_1 => unit());
full_test!(block_last_is_result_2 => str("okay"));
full_test!(block_last_is_result_3 => str("done"));

full_test!(let_is_unit_1 => unit());
full_test!(let_is_unit_2 => unit());

full_test!(let_deconstruct_variable_1 => int(999));
full_test!(let_deconstruct_variable_2 => bun([int(777), str("y")]));
full_test!(let_deconstruct_variable_3 => bun([int(888), str("yyy"), float(5.55)]));
full_test!(let_deconstruct_tuple_1 => bun([bool(true), bool(false)]));
full_test!(let_deconstruct_tuple_2 => bun([int(1), int(2), int(3), str("yy"), str("zz"), str("ww")]));
full_test!(let_deconstruct_tuple_3 => bun([str("a"), int(1), bun([str("c"), float(0.42)]), bool(true), int(2), float(4.0), str("zd")]));

// ------------------ REPORT TESTS ------------------

sem_rep_test!(unknown_binding_global);

#[derive(Default)]
struct ReportLabelReplacer {
    expected_spans: Vec<Span>,
}

impl regex::Replacer for &mut ReportLabelReplacer {
    fn replace_append<'b>(&mut self, cap: &regex::Captures, dst: &mut String) {
        let full_match = cap.get(0).unwrap();
        let snippet = cap.get(1).unwrap().as_str();
        dst.push_str(snippet); // actual replacement here

        let start = full_match.start();
        let end = start + snippet.len();
        let expected_span = Span::new(start, end);
        self.expected_spans.push(expected_span); // store the expected span
    }
}

// --------------------------------------------------

fn bool(b: bool) -> exe::Value {
    exe::Value::Bool(b)
}

fn int(n: i64) -> exe::Value {
    exe::Value::Int(n)
}

fn float(f: f64) -> exe::Value {
    exe::Value::Float(f)
}

fn str(s: &str) -> exe::Value {
    exe::Value::String(s.to_string())
}

fn bun<const N: usize>(items: [exe::Value; N]) -> exe::Value {
    exe::Value::Bundle(Box::new(items))
}

fn unit() -> exe::Value {
    bun([])
}

fn func<const N: usize>(captured: [exe::Value; N]) -> exe::Value {
    bun([exe::Value::Func, bun(captured)])
}

// --------------------------------------------------

fn test_full_program(path: impl AsRef<Path>, expected: exe::Value) {
    let mut compiler = com::init();
    compiler.add_file(path);

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

    let value = exe::run_bytecode(&bytecode);
    assert_eq!(value, expected);
}

macro_rules! full_test {
    (
        $test_name:ident => $expected:expr
    ) => {
        #[test]
        fn $test_name() {
            let path = format!("./tests/lang/{}.mar", stringify!($test_name));
            test_full_program(path, $expected);
        }
    };
}

fn report_test(path: impl AsRef<Path>) {
    let header_regex = Regex::new(r#"--- \[(\w+)\]\: "([^"]*)""#).unwrap();
    let label_regex = Regex::new(r"\\\|([^|]*)\|").unwrap();

    // read test file
    let pseudo_source = std::fs::read_to_string(&path).expect("couldn't read test source file");

    // match expected header
    let header_capture = header_regex
        .captures(&pseudo_source)
        .expect("failed to match report header in test file");
    let expected_header_name = header_capture.get(1).unwrap().as_str();
    let expected_header_msg = header_capture.get(2).unwrap().as_str();

    // replace label syntax
    let mut replacer = ReportLabelReplacer::default();
    let processed_source = label_regex
        .replace_all(&pseudo_source, &mut replacer)
        .to_string();

    // parse and check program
    let mut compiler = com::init();
    compiler.add_source(path.as_ref().display(), &processed_source);
    let compiler = compiler.read_sources().parse();
    assert!(
        compiler.reports.is_empty(),
        "processed semantic report test source has syntax errors"
    );
    let compiler = compiler.check();

    use codespan_reporting::term::{self, termcolor::ColorChoice};
    let color = ColorChoice::AlwaysAnsi;
    let config = term::Config::default();
    compiler
        .emit_reports(color, &config)
        .expect("failed to emit reports");

    assert_eq!(
        compiler.reports.len(),
        1,
        "semantic test case must only emit one report"
    );

    // check header name and message
    let report = &compiler.reports[0];
    assert_eq!(report.header.name(), expected_header_name);
    assert_eq!(report.header.msg(), expected_header_msg);

    // check label spans and count
    let mut primary_label_count = 0;
    for (_, loc, severity) in &report.labels {
        if !matches!(severity, LabelStyle::Primary) {
            continue;
        }

        let lexeme = loc.span.lexeme(&processed_source);
        let span = loc.span;
        primary_label_count += 1;
        assert!(
            replacer.expected_spans.contains(&span),
            "report label for '{lexeme}' was unexpected"
        );
    }
    assert_eq!(
        replacer.expected_spans.len(),
        primary_label_count,
        "primary label count mismatch in report"
    );
}

macro_rules! sem_rep_test {
    (
        $test_name:ident
    ) => {
        #[test]
        fn $test_name() {
            let path = format!("./tests/report/semantic/{}.mar", stringify!($test_name));
            report_test(path);
        }
    };
}

use full_test;
use sem_rep_test;

use codespan_reporting::diagnostic::LabelStyle;
use regex::Regex;

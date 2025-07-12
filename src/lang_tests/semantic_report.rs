sem_rep_test!(unknown_binding_global);
sem_rep_test!(unknown_binding_local);
sem_rep_test!(invalid_integer_too_big);

sem_rep_test!(invalid_expression_record);
sem_rep_test!(invalid_expression_record_big);
sem_rep_test!(invalid_expression_have);
sem_rep_test!(invalid_expression_have_big);
sem_rep_test!(invalid_expression_union);
sem_rep_test!(invalid_expression_union_big);
sem_rep_test!(invalid_expression_class);
sem_rep_test!(invalid_expression_class_big);
sem_rep_test!(invalid_expression_let);
sem_rep_test!(invalid_expression_let_big);
sem_rep_test!(invalid_expression_alias);
sem_rep_test!(invalid_expression_alias_big);
sem_rep_test!(invalid_expression_super);

sem_rep_test!(invalid_super_accessor_class);
sem_rep_test!(invalid_super_accessor_record);
sem_rep_test!(invalid_super_accessor_union);

sem_rep_test!(invalid_type_block);
sem_rep_test!(invalid_type_bool);
sem_rep_test!(invalid_type_break);
sem_rep_test!(invalid_type_empty_array);
sem_rep_test!(invalid_type_float);
sem_rep_test!(invalid_type_if);
sem_rep_test!(invalid_type_int);
sem_rep_test!(invalid_type_loop);
sem_rep_test!(invalid_type_match);
sem_rep_test!(invalid_type_skip);
sem_rep_test!(invalid_type_string);
sem_rep_test!(invalid_type_while);

sem_rep_test!(invalid_type_arg_int);
sem_rep_test!(invalid_type_arg_float);
sem_rep_test!(invalid_type_arg_block);

// ------------------------------------------------------------------------

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

use std::path::Path;

use sem_rep_test;

use codespan_reporting::diagnostic::LabelStyle;
use regex::Regex;

use crate::com::{self, loc::Span};

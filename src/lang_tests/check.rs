test!(type_expr_var);
test!(type_expr_int);
test!(type_expr_float);
test!(type_expr_string);
test!(type_expr_bool);
test!(type_expr_parenthesized);
test!(type_expr_tuple_0);
test!(type_expr_tuple_2);
test!(type_expr_array);
test!(type_expr_lambda_0);
test!(type_expr_lambda_1);
test!(type_expr_lambda_2);
test!(type_expr_lambda_curry);

test!(type_unify_array_00);
test!(type_unify_array_01);
test!(type_unify_array_10);
test!(type_unify_array_11);
test!(type_unify_array);
test!(type_unify_bool);
test!(type_unify_float);
test!(type_unify_int);
test!(type_unify_lambda_0);
test!(type_unify_lambda_1);
test!(type_unify_lambda_2);
test!(type_unify_lambda_curry);
test!(type_unify_string);
test!(type_unify_tuple_0);
test!(type_unify_tuple_2);
test!(type_unify_tuple_3);
test!(type_unify_record_simple);
test!(type_unify_record_generic);
test!(type_unify_union_simple_aa);
test!(type_unify_union_simple_ab);
test!(type_unify_union_simple_ba);
test!(type_unify_union_simple_bb);
test!(type_unify_union_generic_aa);
test!(type_unify_union_generic_ab);
test!(type_unify_union_generic_ba);
test!(type_unify_union_generic_bb);

test!(type_check_array_0);
test!(type_check_array_1);
test!(type_check_array_2);
test!(type_check_do_items);
test!(type_check_if_guard);
test!(type_check_if_items);
test!(type_check_loop_items);
test!(type_check_while_guard);
test!(type_check_while_items);

// ------------------------------------------------------------------------

fn test_program_checks(path: impl AsRef<Path>) {
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
}

macro_rules! test {
    (
        $test_name:ident
    ) => {
        #[test]
        fn $test_name() {
            let path = format!("./tests/check/{}.mar", stringify!($test_name));
            test_program_checks(path);
        }
    };
}

use test;

use crate::com;
use std::path::Path;

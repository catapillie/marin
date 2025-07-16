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
full_test!(block_let_frame => bun([int(1), int(2), int(3)]));

full_test!(let_is_unit_1 => unit());
full_test!(let_is_unit_2 => unit());

full_test!(let_deconstruct_variable_1 => int(999));
full_test!(let_deconstruct_variable_2 => bun([int(777), str("y")]));
full_test!(let_deconstruct_variable_3 => bun([int(888), str("yyy"), float(5.55)]));
full_test!(let_deconstruct_tuple_1 => bun([bool(true), bool(false)]));
full_test!(let_deconstruct_tuple_2 => bun([int(1), int(2), int(3), str("yy"), str("zz"), str("ww")]));
full_test!(let_deconstruct_tuple_3 => bun([str("a"), int(1), bun([str("c"), float(0.42)]), bool(true), int(2), float(4.0), str("zd")]));

full_test!(break_block => int(42));
full_test!(break_block_label => int(42));
full_test!(break_block_nested => int(3));
full_test!(break_block_nested_label => int(3));
full_test!(break_block_nested_early => int(2));
full_test!(break_block_nested_early_label => int(2));
full_test!(break_block_unit => unit());

full_test!(if_true => str("true"));
full_test!(if_false => str("false"));
full_test!(if_true_frame => str("true"));
full_test!(if_false_frame => str("false"));
full_test!(if_unexhaustive_true => unit());
full_test!(if_unexhaustive_false => unit());
full_test!(if_break_true => str("break"));
full_test!(if_break_false => str("break"));
full_test!(if_break_nested_true => str("break"));
full_test!(if_break_nested_false => str("break"));

full_test!(while_is_unit => unit());
full_test!(while_exhaustive_break => str("break"));
full_test!(while_exhaustive_else => str("else"));
full_test!(while_exhaustive_label_break => str("break"));
full_test!(while_exhaustive_label_else => str("else"));
full_test!(while_exhaustive_nested_break => str("break"));
full_test!(while_exhaustive_nested_else => str("else"));

full_test!(loop_break_unit => unit());
full_test!(loop_break_val => str("val"));
full_test!(loop_break_nested_unit => unit());
full_test!(loop_break_nested_val => str("val"));

full_test!(fun_int => int(42));
full_test!(fun_unit => unit());

full_test!(fun_deconstruct_arg_1 => str("a"));
full_test!(fun_deconstruct_arg_2 => str("b"));
full_test!(fun_deconstruct_arg_3 => str("c"));
full_test!(fun_deconstruct_tuple_1 => bun([str("a"), str("b")]));
full_test!(fun_deconstruct_tuple_2 => bun([int(1), int(2), int(3), int(4)]));
full_test!(fun_deconstruct_tuple_3 => bun([str("a"), int(1), bun([str("c"), float(0.42)])]));

full_test!(fun_capture_var_1 => int(42));
full_test!(fun_capture_var_2 => bun([int(42), int(43)]));
full_test!(fun_capture_var_3 => bun([int(42), int(43), int(44), int(45)]));
full_test!(fun_capture_fun_1 => int(42));
full_test!(fun_capture_fun_2 => bun([int(42), int(43)]));
full_test!(fun_capture_fun_3 => bun([int(42), int(43), bun([int(44), int(44)]), int(45)]));

full_test!(fun_curry_1 => int(42));
full_test!(fun_curry_2 => bun([str("a"), str("b")]));
full_test!(fun_curry_3 => bun([int(1), int(2), int(3), int(4)]));
full_test!(fun_curry_capture_var_1 => int(42));
full_test!(fun_curry_capture_var_2 => bun([str("h"), bun([int(42), int(43)])]));
full_test!(fun_curry_capture_var_3 => bun([str("h"), bun([int(42), int(43), int(44), int(45)]), str("t")]));
full_test!(fun_curry_capture_fun_1 => int(42));
full_test!(fun_curry_capture_fun_2 => bun([str("h"), bun([int(42), int(43)])]));
full_test!(fun_curry_capture_fun_3 => bun([str("h"), bun([int(42), int(43), bun([int(44), int(44)]), int(45)]), str("t")]));

full_test!(let_fun_1 => int(42));
full_test!(let_fun_2 => bun([int(42), str("43")]));
full_test!(let_fun_3 => bun([int(1), int(2), int(3), int(4)]));
full_test!(let_fun_4 => bun([bun([int(8), int(8)]), bun([int(8), int(8)]), bun([int(8), int(8)])]));
full_test!(let_fun_5 => bun([int(42), int(42)]));
full_test!(let_fun_6 => bun([int(12), int(12)]));

full_test!(let_generalize_1 => bun([int(42), str("a"), float(12.3)]));
full_test!(let_generalize_2 => bun([bun([int(2), float(1.0)]), bun([bool(true), str("a")]), bun([int(0), func([])])]));
full_test!(let_generalize_3 => bun([str("u"), int(42), str("a")]));

full_test!(array_index_1 => int(1));
full_test!(array_index_2 => int(3));
full_test!(array_index_3 => int(43));

full_test!(record_empty => record([]));
full_test!(record_fields => record([float(42.0), float(22.2)]));
full_test!(record_generic_1 => bun([record([float(42.0), float(22.2)]), record([int(42), int(22)]), record([str("a"), str("b")])]));
full_test!(record_generic_2 => bun([record([int(1), int(2)]), record([int(42), float(22.2)]), record([str("a"), bool(true)])]));
full_test!(record_getter => bun([float(42.0), float(22.2)]));

full_test!(union_variants_empty_a => union(0, []));
full_test!(union_variants_empty_b => union(1, []));
full_test!(union_variants_fields_a => union(0, [int(42)]));
full_test!(union_variants_fields_b => union(1, [float(81.7)]));
full_test!(union_variants_recursive_a => union(0, []));
full_test!(union_variants_recursive_ba => union(1, [union(0, [])]));
full_test!(union_variants_recursive_bba => union(1, [union(1, [union(0, [])])]));
full_test!(union_generic_1_a_1 => union(0, [str("c")]));
full_test!(union_generic_1_a_2 => union(0, [int(3)]));
full_test!(union_generic_1_b_1 => union(1, [str("a"), str("b")]));
full_test!(union_generic_1_b_2 => union(1, [int(1), int(2)]));
full_test!(union_generic_2_a => union(0, [int(1)]));
full_test!(union_generic_2_b => union(1, [unit()]));
full_test!(union_generic_2_ab => union(2, [str("3"), int(4)]));

full_test!(type_expr_var => unit());
full_test!(type_expr_int => unit());
full_test!(type_expr_float => unit());
full_test!(type_expr_string => unit());
full_test!(type_expr_bool => unit());
full_test!(type_expr_parenthesized => unit());
full_test!(type_expr_tuple_0 => unit());
full_test!(type_expr_tuple_2 => unit());
full_test!(type_expr_array => unit());
full_test!(type_expr_lambda_0 => unit());
full_test!(type_expr_lambda_1 => unit());
full_test!(type_expr_lambda_2 => unit());
full_test!(type_expr_lambda_curry => unit());

// ------------------------------------------------------------------------

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

use full_test;

use super::*;
use crate::{com, exe};
use std::path::Path;

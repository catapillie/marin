test!(empty_program_returns_unit => unit());
test!(empty_program_returns_unit_explicitly => unit());

test!(returns_true => bool(true));
test!(returns_false => bool(false));
test!(returns_integer => int(42));
test!(returns_float => float(4.87));
test!(returns_string => str("hello, world"));
test!(returns_func => func([]));

test!(returns_tuple_1 => bun([int(8), float(0.5), str("hi")]));
test!(returns_tuple_2 => bun([bun([bool(true), float(2.0)]), bun([int(3), float(4.0), int(5)]), str("6")]));

test!(returns_array_1 => bun([int(1), int(2), int(3), int(4)]));
test!(returns_array_2 => bun([bun([int(1), int(2)]), bun([]), bun([int(3)]), bun([int(4), int(5), int(6)])]));

test!(block_empty_is_unit => unit());
test!(block_singleton_1 => unit());
test!(block_singleton_2 => int(73));
test!(block_singleton_3 => bun([str("single"), bun([bool(true), str("block")])]));
test!(block_last_is_result_1 => unit());
test!(block_last_is_result_2 => str("okay"));
test!(block_last_is_result_3 => str("done"));
test!(block_let_frame => bun([int(1), int(2), int(3)]));

test!(let_is_unit_1 => unit());
test!(let_is_unit_2 => unit());

test!(let_deconstruct_variable_1 => int(999));
test!(let_deconstruct_variable_2 => bun([int(777), str("y")]));
test!(let_deconstruct_variable_3 => bun([int(888), str("yyy"), float(5.55)]));
test!(let_deconstruct_tuple_1 => bun([bool(true), bool(false)]));
test!(let_deconstruct_tuple_2 => bun([int(1), int(2), int(3), str("yy"), str("zz"), str("ww")]));
test!(let_deconstruct_tuple_3 => bun([str("a"), int(1), bun([str("c"), float(0.42)]), bool(true), int(2), float(4.0), str("zd")]));

test!(break_block => int(42));
test!(break_block_label => int(42));
test!(break_block_nested => int(3));
test!(break_block_nested_label => int(3));
test!(break_block_nested_early => int(2));
test!(break_block_nested_early_label => int(2));
test!(break_block_unit => unit());

test!(if_true => str("true"));
test!(if_false => str("false"));
test!(if_true_frame => str("true"));
test!(if_false_frame => str("false"));
test!(if_unexhaustive_true => unit());
test!(if_unexhaustive_false => unit());
test!(if_break_true => str("break"));
test!(if_break_false => str("break"));
test!(if_break_nested_true => str("break"));
test!(if_break_nested_false => str("break"));

test!(while_is_unit => unit());
test!(while_exhaustive_break => str("break"));
test!(while_exhaustive_else => str("else"));
test!(while_exhaustive_label_break => str("break"));
test!(while_exhaustive_label_else => str("else"));
test!(while_exhaustive_nested_break => str("break"));
test!(while_exhaustive_nested_else => str("else"));

test!(loop_break_unit => unit());
test!(loop_break_val => str("val"));
test!(loop_break_nested_unit => unit());
test!(loop_break_nested_val => str("val"));

test!(fun_int => int(42));
test!(fun_unit => unit());

test!(fun_deconstruct_arg_1 => str("a"));
test!(fun_deconstruct_arg_2 => str("b"));
test!(fun_deconstruct_arg_3 => str("c"));
test!(fun_deconstruct_tuple_1 => bun([str("a"), str("b")]));
test!(fun_deconstruct_tuple_2 => bun([int(1), int(2), int(3), int(4)]));
test!(fun_deconstruct_tuple_3 => bun([str("a"), int(1), bun([str("c"), float(0.42)])]));

test!(fun_capture_var_1 => int(42));
test!(fun_capture_var_2 => bun([int(42), int(43)]));
test!(fun_capture_var_3 => bun([int(42), int(43), int(44), int(45)]));
test!(fun_capture_fun_1 => int(42));
test!(fun_capture_fun_2 => bun([int(42), int(43)]));
test!(fun_capture_fun_3 => bun([int(42), int(43), bun([int(44), int(44)]), int(45)]));

test!(fun_curry_1 => int(42));
test!(fun_curry_2 => bun([str("a"), str("b")]));
test!(fun_curry_3 => bun([int(1), int(2), int(3), int(4)]));
test!(fun_curry_capture_var_1 => int(42));
test!(fun_curry_capture_var_2 => bun([str("h"), bun([int(42), int(43)])]));
test!(fun_curry_capture_var_3 => bun([str("h"), bun([int(42), int(43), int(44), int(45)]), str("t")]));
test!(fun_curry_capture_fun_1 => int(42));
test!(fun_curry_capture_fun_2 => bun([str("h"), bun([int(42), int(43)])]));
test!(fun_curry_capture_fun_3 => bun([str("h"), bun([int(42), int(43), bun([int(44), int(44)]), int(45)]), str("t")]));

test!(let_fun_1 => int(42));
test!(let_fun_2 => bun([int(42), str("43")]));
test!(let_fun_3 => bun([int(1), int(2), int(3), int(4)]));
test!(let_fun_4 => bun([bun([int(8), int(8)]), bun([int(8), int(8)]), bun([int(8), int(8)])]));
test!(let_fun_5 => bun([int(42), int(42)]));
test!(let_fun_6 => bun([int(12), int(12)]));

test!(let_generalize_1 => bun([int(42), str("a"), float(12.3)]));
test!(let_generalize_2 => bun([bun([int(2), float(1.0)]), bun([bool(true), str("a")]), bun([int(0), func([])])]));
test!(let_generalize_3 => bun([str("u"), int(42), str("a")]));

test!(array_index_1 => int(1));
test!(array_index_2 => int(3));
test!(array_index_3 => int(43));

test!(record_empty => record([]));
test!(record_fields => record([float(42.0), float(22.2)]));
test!(record_generic_1 => bun([record([float(42.0), float(22.2)]), record([int(42), int(22)]), record([str("a"), str("b")])]));
test!(record_generic_2 => bun([record([int(1), int(2)]), record([int(42), float(22.2)]), record([str("a"), bool(true)])]));
test!(record_getter => bun([float(42.0), float(22.2)]));

test!(union_variants_empty_a => union(0, []));
test!(union_variants_empty_b => union(1, []));
test!(union_variants_fields_a => union(0, [int(42)]));
test!(union_variants_fields_b => union(1, [float(81.7)]));
test!(union_variants_recursive_a => union(0, []));
test!(union_variants_recursive_ba => union(1, [union(0, [])]));
test!(union_variants_recursive_bba => union(1, [union(1, [union(0, [])])]));
test!(union_generic_1_a_1 => union(0, [str("c")]));
test!(union_generic_1_a_2 => union(0, [int(3)]));
test!(union_generic_1_b_1 => union(1, [str("a"), str("b")]));
test!(union_generic_1_b_2 => union(1, [int(1), int(2)]));
test!(union_generic_2_a => union(0, [int(1)]));
test!(union_generic_2_b => union(1, [unit()]));
test!(union_generic_2_ab => union(2, [str("3"), int(4)]));

test!(match_empty_is_unit => unit());
test!(match_binding => int(42));
test!(match_int_fallback => int(2));
test!(match_int_success => int(42));
test!(match_tuple_deconstruct => bun([int(43), int(42)]));
test!(match_tuple_left_fallback => bun([int(2), int(2)]));
test!(match_tuple_left_success => bun([int(42), int(2)]));
test!(match_tuple_right_fallback => bun([int(2), int(2)]));
test!(match_tuple_right_success => bun([int(2), int(42)]));
test!(match_tuple_both_fallback_both => bun([int(2), int(2)]));
test!(match_tuple_both_fallback_left => bun([int(2), int(1)]));
test!(match_tuple_both_fallback_right => bun([int(1), int(2)]));
test!(match_tuple_both_success => bun([int(42), int(42)]));
test!(match_tuple_nested_deconstruct => bun([int(1), int(42), int(43)]));
test!(match_tuple_nested_test_fallback => bun([int(1), int(44), int(43)]));
test!(match_tuple_nested_test_success => bun([int(1), int(999), int(43)]));
test!(match_union_exhaustive_a => str("a"));
test!(match_union_exhaustive_b => str("b"));
test!(match_union_deconstruct_1_a => str("a"));
test!(match_union_deconstruct_1_b => str("sss"));
test!(match_union_deconstruct_2_a => bun([str("a1"), str("a2")]));
test!(match_union_deconstruct_2_b => bun([str("s1"), str("s2")]));
test!(match_record_deconstruct => bun([int(42), float(1.0)]));
test!(match_record_test_x_fallback => bun([int(44), float(1.0)]));
test!(match_record_test_x_success => bun([int(999), float(1.0)]));
test!(match_record_test_y_fallback => bun([int(44), float(1.0)]));
test!(match_record_test_y_success => bun([int(44), float(999.0)]));
test!(match_record_test_both_fallback_both => bun([int(1), float(1.0)]));
test!(match_record_test_both_fallback_x => bun([int(1), float(42.0)]));
test!(match_record_test_both_fallback_y => bun([int(42), float(1.0)]));
test!(match_record_test_both_success => bun([int(999), float(999.0)]));

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

macro_rules! test {
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

use test;

use super::*;
use crate::{com, exe};
use std::path::Path;

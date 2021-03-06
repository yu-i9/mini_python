extern crate core;

use core::error::pyerr_occurred;
use core::utils::*;

macro_rules! test_cases {
    ( $( $i:ident ), * ) => {
        $(
            #[test]
            fn $i() {
                run(&["tests/tests/", stringify!($i), ".py"].join(""));
                assert!(!pyerr_occurred())
            }
        )*
    }
}

#[test]
fn assert_true() {
    run_prog_string("assert 42 == 42\n".to_string());
    assert!(!pyerr_occurred())
}

#[test]
fn assert_false() {
    run_prog_string("assert 1 == 42\n".to_string());
    assert!(pyerr_occurred())
}

test_cases![
    blank_lines, parse_string, consecutive_call, if_false, if_true,
    while_normal, while_continue, while_break,
    def, def_argument, def_recursive, def_internal, def_ho, def_lexical_scope,
    class_var, class_instance_var, class_method, assign_attr, class_update, class_init,
    dict_basic,
    string_add, meta_add, inherit_meta_add, meta_add_lazy,
    list_basic, list_append,
    builtin_len,
    inheritance_simple, inheritance_complex,
    bool_arith,
    type_call,
    for_stmt,
    try_catch_basic, try_catch_loop, try_catch_fun, catch_type_error
];

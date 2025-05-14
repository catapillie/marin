#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Builtin {
    int_add,
    int_sub,
    int_mul,
    int_div,
    int_mod,

    float_add,
    float_sub,
    float_mul,
    float_div,
    float_mod,

    string_concat,
}

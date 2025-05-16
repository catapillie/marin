use std::fmt::Display;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Builtin {
    int_add,
    int_sub,
    int_mul,
    int_div,
    int_mod,
    int_and,
    int_or,
    int_xor,
    int_eq,
    int_ne,
    int_lt,
    int_le,
    int_gt,
    int_ge,

    float_add,
    float_sub,
    float_mul,
    float_div,
    float_mod,
    float_eq,
    float_ne,
    float_lt,
    float_le,
    float_gt,
    float_ge,

    string_concat,
    string_eq,
    string_ne,
    string_lt,
    string_le,
    string_gt,
    string_ge,

    bool_and,
    bool_or,
    bool_xor,
    bool_eq,
    bool_ne,

    pow,
    exp,
    ln,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::int_add => write!(f, "int_add"),
            Self::int_sub => write!(f, "int_sub"),
            Self::int_mul => write!(f, "int_mul"),
            Self::int_div => write!(f, "int_div"),
            Self::int_mod => write!(f, "int_mod"),
            Self::int_and => write!(f, "int_and"),
            Self::int_or => write!(f, "int_or"),
            Self::int_xor => write!(f, "int_xor"),
            Self::int_eq => write!(f, "int_eq"),
            Self::int_ne => write!(f, "int_ne"),
            Self::int_lt => write!(f, "int_lt"),
            Self::int_le => write!(f, "int_le"),
            Self::int_gt => write!(f, "int_gt"),
            Self::int_ge => write!(f, "int_ge"),

            Self::float_add => write!(f, "float_add"),
            Self::float_sub => write!(f, "float_sub"),
            Self::float_mul => write!(f, "float_mul"),
            Self::float_div => write!(f, "float_div"),
            Self::float_mod => write!(f, "float_mod"),
            Self::float_eq => write!(f, "float_eq"),
            Self::float_ne => write!(f, "float_ne"),
            Self::float_lt => write!(f, "float_lt"),
            Self::float_le => write!(f, "float_le"),
            Self::float_gt => write!(f, "float_gt"),
            Self::float_ge => write!(f, "float_ge"),

            Self::string_concat => write!(f, "string_concat"),
            Self::string_eq => write!(f, "string_eq"),
            Self::string_ne => write!(f, "string_ne"),
            Self::string_lt => write!(f, "string_lt"),
            Self::string_le => write!(f, "string_le"),
            Self::string_gt => write!(f, "string_gt"),
            Self::string_ge => write!(f, "string_ge"),

            Self::bool_and => write!(f, "bool_and"),
            Self::bool_or => write!(f, "bool_or"),
            Self::bool_xor => write!(f, "bool_xor"),
            Self::bool_eq => write!(f, "bool_eq"),
            Self::bool_ne => write!(f, "bool_ne"),

            Self::pow => write!(f, "pow"),
            Self::exp => write!(f, "exp"),
            Self::ln => write!(f, "ln"),
        }
    }
}

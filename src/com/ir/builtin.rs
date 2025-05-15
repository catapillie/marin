use std::fmt::Display;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

    pow,
    exp,
    ln,

    string_concat,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::int_add => write!(f, "int_add"),
            Self::int_sub => write!(f, "int_sub"),
            Self::int_mul => write!(f, "int_mul"),
            Self::int_div => write!(f, "int_div"),
            Self::int_mod => write!(f, "int_mod"),

            Self::float_add => write!(f, "float_add"),
            Self::float_sub => write!(f, "float_sub"),
            Self::float_mul => write!(f, "float_mul"),
            Self::float_div => write!(f, "float_div"),
            Self::float_mod => write!(f, "float_mod"),

            Self::pow => write!(f, "pow"),
            Self::exp => write!(f, "exp"),
            Self::ln => write!(f, "ln"),

            Self::string_concat => write!(f, "string_concat"),
        }
    }
}

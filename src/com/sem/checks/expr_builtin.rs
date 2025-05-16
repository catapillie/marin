use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Report},
};

use ir::Type as Ty;

impl Checker<'_, '_> {
    #[rustfmt::skip]
    pub fn check_builtin(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let span = e.span;
        let name = &e.span.lexeme(self.source)[1..];
        match name {
            // operations
            "int_add" => builtin_func!(self, span, int_add :: int, int -> int),
            "int_sub" => builtin_func!(self, span, int_sub :: int, int -> int),
            "int_mul" => builtin_func!(self, span, int_mul :: int, int -> int),
            "int_div" => builtin_func!(self, span, int_div :: int, int -> int),
            "int_mod" => builtin_func!(self, span, int_mod :: int, int -> int),
            "int_and" => builtin_func!(self, span, int_and :: int, int -> int),
            "int_or" => builtin_func!(self, span, int_or :: int, int -> int),
            "int_xor" => builtin_func!(self, span, int_xor :: int, int -> int),
            "int_eq" => builtin_func!(self, span, int_eq :: int, int -> bool),
            "int_ne" => builtin_func!(self, span, int_ne :: int, int -> bool),
            "int_lt" => builtin_func!(self, span, int_lt :: int, int -> bool),
            "int_le" => builtin_func!(self, span, int_le :: int, int -> bool),
            "int_gt" => builtin_func!(self, span, int_gt :: int, int -> bool),
            "int_ge" => builtin_func!(self, span, int_ge :: int, int -> bool),
            "int_pos" => builtin_func!(self, span, int_pos :: int -> int),
            "int_neg" => builtin_func!(self, span, int_neg :: int -> int),
            "int_not" => builtin_func!(self, span, int_not :: int -> int),

            "float_add" => builtin_func!(self, span, float_add :: float, float -> float),
            "float_sub" => builtin_func!(self, span, float_sub :: float, float -> float),
            "float_mul" => builtin_func!(self, span, float_mul :: float, float -> float),
            "float_div" => builtin_func!(self, span, float_div :: float, float -> float),
            "float_mod" => builtin_func!(self, span, float_mod :: float, float -> float),
            "float_eq" => builtin_func!(self, span, float_eq :: float, float -> bool),
            "float_ne" => builtin_func!(self, span, float_ne :: float, float -> bool),
            "float_lt" => builtin_func!(self, span, float_lt :: float, float -> bool),
            "float_le" => builtin_func!(self, span, float_le :: float, float -> bool),
            "float_gt" => builtin_func!(self, span, float_gt :: float, float -> bool),
            "float_ge" => builtin_func!(self, span, float_ge :: float, float -> bool),
            "float_pos" => builtin_func!(self, span, float_pos :: float -> float),
            "float_neg" => builtin_func!(self, span, float_neg :: float -> float),

            "string_concat" => builtin_func!(self, span, string_concat :: string, string -> string),
            "string_eq" => builtin_func!(self, span, string_eq :: string, string -> bool),
            "string_ne" => builtin_func!(self, span, string_ne :: string, string -> bool),
            "string_lt" => builtin_func!(self, span, string_lt :: string, string -> bool),
            "string_le" => builtin_func!(self, span, string_le :: string, string -> bool),
            "string_gt" => builtin_func!(self, span, string_gt :: string, string -> bool),
            "string_ge" => builtin_func!(self, span, string_ge :: string, string -> bool),

            "bool_and" => builtin_func!(self, span, bool_and :: bool, bool -> bool),
            "bool_or" => builtin_func!(self, span, bool_or :: bool, bool -> bool),
            "bool_xor" => builtin_func!(self, span, bool_xor :: bool, bool -> bool),
            "bool_eq" => builtin_func!(self, span, bool_eq :: bool, bool -> bool),
            "bool_ne" => builtin_func!(self, span, bool_ne :: bool, bool -> bool),
            "bool_not" => builtin_func!(self, span, bool_not :: bool -> bool),

            "pow" => builtin_func!(self, span, pow :: float, float -> float),
            "exp" => builtin_func!(self, span, exp :: float -> float),
            "ln" => builtin_func!(self, span, ln :: float -> float),
            "sin" => builtin_func!(self, span, sin :: float -> float),
            "cos" => builtin_func!(self, span, cos :: float -> float),
            "tan" => builtin_func!(self, span, tan :: float -> float),
            "asin" => builtin_func!(self, span, asin :: float -> float),
            "acos" => builtin_func!(self, span, acos :: float -> float),
            "atan" => builtin_func!(self, span, atan :: float -> float),

            "panic" => {
                let arg_ty = self.create_fresh_type(None);
                let ret_ty = self.create_fresh_type(None);
                (
                    ir::Expr::Builtin(ir::Builtin::panic),
                    self.create_type(
                        Ty::Lambda(Box::new([arg_ty]), ret_ty),
                        Some(span),
                    )
                )
            },

            _ => {
                self.reports.push(
                    Report::error(Header::InvalidBuiltin(name.to_string()))
                        .with_primary_label(Label::Empty, span.wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }
}

macro_rules! builtin_func {
    (
        $self:ident, $span:ident,
        $builtin:ident
        ::
        $($arg_ty:ident),+ -> $ret_ty:ident
    ) => {
        (
            ir::Expr::Builtin(ir::Builtin::$builtin),
            $self.create_type(
                Ty::Lambda(
                    Box::new([
                        $(
                            $self.native_types.$arg_ty
                        ),+
                    ]),
                    $self.native_types.$ret_ty,
                ),
                Some($span),
            )
        )
    };
}

use builtin_func;

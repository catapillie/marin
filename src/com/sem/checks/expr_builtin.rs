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

            "float_add" => builtin_func!(self, span, float_add :: float, float -> float),
            "float_sub" => builtin_func!(self, span, float_sub :: float, float -> float),
            "float_mul" => builtin_func!(self, span, float_mul :: float, float -> float),
            "float_div" => builtin_func!(self, span, float_div :: float, float -> float),
            "float_mod" => builtin_func!(self, span, float_mod :: float, float -> float),

            "pow" => builtin_func!(self, span, pow :: float, float -> float),
            "exp" => builtin_func!(self, span, exp :: float -> float),
            "ln" => builtin_func!(self, span, ln :: float -> float),

            "string_concat" => builtin_func!(self, span, string_concat :: string, string -> string),

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

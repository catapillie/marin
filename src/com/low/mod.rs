use super::ir;

enum Expr {
    Int { val: i64 },
    Float { val: f64 },
    String { val: String },
    Bool { val: bool },
    Tuple { items: Box<[Expr]> },
    Array { items: Box<[Expr]> },
}

impl Expr {
    fn unit() -> Self {
        Self::Tuple {
            items: Box::new([]),
        }
    }
}

enum Stmt {
    Expr { expr: Box<Expr> },
}

pub fn lower(mut modules: Vec<ir::Module>, entities: ir::Entities, dependency_order: Vec<usize>) {
    let mut stmts = Vec::new();
    for file_id in dependency_order {
        let file_stmts = std::mem::take(&mut modules[file_id].stmts);
        stmts.extend_from_slice(&file_stmts);
    }
}

fn lower_statement_block(stmts: impl Iterator<Item = ir::Stmt>) -> Vec<Stmt> {
    use ir::Stmt as S;
    stmts
        .filter_map(|stmt| match stmt {
            S::Missing => None,
            S::Nothing => None,
            S::Expr { expr, ty: _ } => Some(lower_statement_expression(expr)),
            S::Let { .. } => todo!(""),
        })
        .collect()
}

fn lower_statement_expression(expr: ir::Expr) -> Stmt {
    Stmt::Expr {
        expr: Box::new(lower_expression(expr)),
    }
}

fn lower_expression_block(stmts: impl Iterator<Item = ir::Stmt>) -> Vec<Stmt> {
    use ir::Stmt as S;
    stmts
        .filter_map(|stmt| match stmt {
            S::Missing => None,
            S::Nothing => None,
            S::Expr { .. } => todo!(),
            S::Let { .. } => todo!(""),
        })
        .collect()
}

fn lower_expression(expr: ir::Expr) -> Expr {
    use ir::Expr as E;
    match expr {
        E::Missing => unreachable!("attempt to lower missing expr"),
        E::Int { val } => lower_int(val),
        E::Float { val } => lower_float(val),
        E::String { val } => lower_string(val),
        E::Bool { val } => lower_bool(val),
        E::Var { id } => todo!(),
        E::Tuple { items } => lower_tuple(items),
        E::Array { items } => lower_array(items),
        E::Block { stmts, label } => todo!(),
        E::Conditional {
            branches,
            is_exhaustive,
        } => todo!(),
        E::Break { expr, label } => todo!(),
        E::Skip { label } => todo!(),
        E::Fun {
            name,
            recursive_binding,
            signature,
            expr,
        } => todo!(),
        E::Call { callee, args } => todo!(),
        E::Variant { tag, items } => todo!(),
        E::Record { fields } => todo!(),
        E::Access { accessed, index } => todo!(),
    }
}

fn lower_int(val: i64) -> Expr {
    Expr::Int { val }
}

fn lower_float(val: f64) -> Expr {
    Expr::Float { val }
}

fn lower_string(val: String) -> Expr {
    Expr::String { val }
}

fn lower_bool(val: bool) -> Expr {
    Expr::Bool { val }
}

fn lower_tuple(items: Box<[ir::Expr]>) -> Expr {
    Expr::Tuple {
        items: items.into_iter().map(lower_expression).collect(),
    }
}

fn lower_array(items: Box<[ir::Expr]>) -> Expr {
    Expr::Array {
        items: items.into_iter().map(lower_expression).collect(),
    }
}

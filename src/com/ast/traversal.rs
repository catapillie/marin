use super::{Branch, Expr};

pub fn preorder_traversal(expr: &Expr) -> Vec<&Expr> {
    let mut nodes = Vec::new();
    walk_expr(expr, &mut nodes);
    nodes
}

fn walk_expr<'a>(expr: &'a Expr, nodes: &mut Vec<&'a Expr>) {
    nodes.push(expr);

    use Expr as E;
    match expr {
        E::Missing(..) => {}
        E::Int(..) => {}
        E::Float(..) => {}
        E::String(..) => {}
        E::True(..) => {}
        E::False(..) => {}
        E::Underscores(..) => {}
        E::Var(..) => {}
        E::Tuple(e) => {
            for item in &e.items {
                walk_expr(item, nodes);
            }
        }
        E::Array(e) => {
            for item in &e.items {
                walk_expr(item, nodes);
            }
        }
        E::Spread(..) => {}
        E::Block(e) => {
            for item in &e.items {
                walk_expr(item, nodes);
            }
        }
        E::Conditional(e) => {
            walk_branch(&e.first_branch, nodes);
            for (_, branch) in &e.else_branches {
                walk_branch(branch, nodes);
            }
        }
        E::Break(e) => {
            if let Some(value) = &e.expr {
                walk_expr(value, nodes);
            }
        }
        E::Skip(..) => {}
        E::Call(e) => {
            walk_expr(&e.callee, nodes);
            for arg in &e.args {
                walk_expr(arg, nodes);
            }
        }
        E::Access(e) => walk_expr(&e.accessed, nodes),
        E::Let(e) => {
            walk_expr(&e.pattern, nodes);
            walk_expr(&e.value, nodes);
        }
        E::Pub(e) => {
            walk_expr(&e.expr, nodes);
        }
        E::Fun(e) => {
            walk_expr(&e.signature, nodes);
            walk_expr(&e.value, nodes);
        }
        E::Import(..) => {}
        E::Super(..) => {}
        E::Record(..) => {}
        E::RecordValue(e) => {
            for (name, value) in &e.fields {
                walk_expr(name, nodes);
                if let Some(value) = value {
                    walk_expr(value, nodes);
                }
            }
        }
        E::Union(..) => {}
        E::Class(..) => {}
        E::Have(e) => {
            for item in &e.items {
                walk_expr(item, nodes);
            }
        }
    }
}

fn walk_branch<'a>(branch: &'a Branch, nodes: &mut Vec<&'a Expr>) {
    use Branch as B;
    match branch {
        B::If(b) => {
            walk_expr(&b.condition, nodes);
            for item in &b.body {
                walk_expr(item, nodes);
            }
        }
        B::While(b) => {
            walk_expr(&b.condition, nodes);
            for item in &b.body {
                walk_expr(item, nodes);
            }
        }
        B::Loop(b) => {
            for item in &b.body {
                walk_expr(item, nodes);
            }
        }
        B::Match(b) => {
            walk_expr(&b.scrutinee, nodes);
            for case in &b.cases {
                walk_expr(&case.pattern, nodes);
                walk_expr(&case.value, nodes);
            }
        }
        B::Else(b) => {
            for item in &b.body {
                walk_expr(item, nodes);
            }
        }
    }
}

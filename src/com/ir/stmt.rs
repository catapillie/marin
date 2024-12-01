use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Let,
}

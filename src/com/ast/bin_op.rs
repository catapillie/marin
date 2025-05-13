#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl BinOp {
    /// Higher means more priority
    pub fn precedence(&self) -> usize {
        match self {
            Self::Add => 10,
            Self::Sub => 10,
            Self::Mul => 20,
            Self::Div => 20,
            Self::Mod => 20,
        }
    }
}

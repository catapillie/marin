#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Xor,
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitXor,
    BitOr,
    BitAnd,
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
            Self::Xor => 10,
            Self::Or => 10,
            Self::And => 15,
            Self::Eq => 20,
            Self::Ne => 20,
            Self::Lt => 20,
            Self::Le => 20,
            Self::Gt => 20,
            Self::Ge => 20,
            Self::BitXor => 30,
            Self::BitOr => 30,
            Self::BitAnd => 30,
            Self::Add => 40,
            Self::Sub => 40,
            Self::Mul => 50,
            Self::Div => 50,
            Self::Mod => 50,
        }
    }
}

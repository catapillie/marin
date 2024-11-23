use crate::com::Token;

pub enum Header<'src> {
    Internal(String),
    InvalidCharacterSequence(&'src str),
    ExpectedToken(Token, Token),
    ExpectedExpression(),
    EmptyImport(),
}

impl<'src> Header<'src> {
    #[rustfmt::skip]
    pub fn name(&self) -> &str {
        use Header as H;
        match self {
            H::Internal(..) => "internal",
            H::InvalidCharacterSequence(..) => "invalid_character_sequence",
            H::ExpectedToken(..) => "expected_token",
            H::ExpectedExpression(..) => "expected_expression",
            H::EmptyImport(..) => "empty_import",
        }
    }

    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        use Header as H;
        match self {
            H::Internal(msg)
                => msg.clone(),
            H::InvalidCharacterSequence(seq)
                => format!("invalid characters '{seq}'"),
            H::ExpectedToken(want, have)
                => format!("expected {want}, encounted {have} instead"),
            H::ExpectedExpression()
                => "expected an expression".to_string(),
            H::EmptyImport()
                => "empty import expression".to_string(),
        }
    }
}

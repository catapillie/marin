use crate::com::Token;

pub enum Header<'src> {
    Internal(String),
    InvalidCharacterSequence(&'src str),
    ExpectedToken(Token, Token),
    ExpectedExpression(),
}

impl<'src> Header<'src> {
    #[rustfmt::skip]
    pub fn name(&self) -> &str {
        use Header as R;
        match self {
            R::Internal(..) => "internal",
            R::InvalidCharacterSequence(..) => "invalid_character_sequence",
            R::ExpectedToken(..) => "expected_token",
            R::ExpectedExpression(..) => "expected_expression",
        }
    }

    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        match self {
            Header::Internal(msg)
                => msg.clone(),
            Header::InvalidCharacterSequence(seq)
                => format!("invalid characters '{seq}'"),
            Header::ExpectedToken(want, have)
                => format!("expected {want}, encounted {have} instead"),
            Header::ExpectedExpression()
                => "expected an expression".to_string(),
        }
    }
}

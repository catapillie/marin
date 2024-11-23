use crate::com::Token;

pub enum Header {
    Internal(String),

    CompilerNoInput(),
    CompilerIOPath(String, String),
    CompilerIOFile(String, String),

    InvalidCharacterSequence(String),
    ExpectedToken(Token, Token),
    ExpectedExpression(),
    EmptyImport(),
}

impl Header {
    #[rustfmt::skip]
    pub fn name(&self) -> &str {
        use Header as H;
        match self {
            H::Internal(..) => "internal",
            H::CompilerNoInput(..) => "compiler_no_input",
            H::CompilerIOPath(..) => "compiler_io_path",
            H::CompilerIOFile(..) => "compiler_io_file",
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
            H::CompilerNoInput()
                => "no input files".to_string(),
            H::CompilerIOPath(path, msg)
                => format!("failed to read path '{path}: {msg}"),
            H::CompilerIOFile(path, msg)
                => format!("failed to read file '{path}: {msg}"),
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

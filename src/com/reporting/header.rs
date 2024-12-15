use crate::com::{ir::TypeString, Token};

pub enum Header {
    #[allow(dead_code)]
    Internal(String),

    CompilerNoInput(),
    CompilerNoSuchPath(String),
    CompilerBadExtension(String),
    CompilerIO(String, String),

    InvalidDependencyPath(),
    NoSuchDependency(String),
    UnstagedDependency(String),
    SelfDependency(String),
    DependencyCycle(),
    OutsideDependency(),

    InvalidCharacterSequence(String),
    ExpectedToken(Token, Token),
    ExpectedExpression(),
    EmptyImport(),
    InvalidImportQuery(),
    RedundantSuper(),
    FileReimported(String),
    InvalidInteger(),
    InvalidFloat(),
    InvalidExpression(),
    InvalidPattern(),
    InvalidSignature(),
    InvalidLabel(),
    InvalidBreak(Option<String>),
    InvalidSkip(Option<String>),
    UnknownVariable(String),
    NotVariable(String),
    TypeMismatch(TypeString, TypeString),
    UnreachableConditionalBranches(usize),
    RefutablePattern(),
}

impl Header {
    #[rustfmt::skip]
    pub fn name(&self) -> &str {
        use Header as H;
        match self {
            H::Internal(..) => "internal",

            H::CompilerNoInput(..) => "compiler_no_input",
            H::CompilerNoSuchPath(..) => "compiler_no_such_path",
            H::CompilerBadExtension(..) => "compiler_bad_extension",
            H::CompilerIO(..) => "compiler_io",

            H::InvalidDependencyPath(..) => "invalid_dependency_path",
            H::NoSuchDependency(..) => "no_such_dependency",
            H::UnstagedDependency(..) => "unstaged_dependency",
            H::SelfDependency(..) => "self_dependency",
            H::DependencyCycle(..) => "dependency_cycle",
            H::OutsideDependency(..) => "outside_dependency",

            H::InvalidCharacterSequence(..) => "invalid_character_sequence",
            H::ExpectedToken(..) => "expected_token",
            H::ExpectedExpression(..) => "expected_expression",
            H::EmptyImport(..) => "empty_import",
            H::InvalidImportQuery(..) => "invalid_import_query",
            H::RedundantSuper(..) => "redundant_super",
            H::FileReimported(..) => "file_reimported",
            H::InvalidInteger(..) => "invalid_integer",
            H::InvalidFloat(..) => "invalid_float",
            H::InvalidExpression(..) => "invalid_expression",
            H::InvalidPattern(..) => "invalid_pattern",
            H::InvalidSignature(..) => "invalid_signature",
            H::InvalidLabel(..) => "invalid_label",
            H::InvalidBreak(..) => "invalid_break",
            H::InvalidSkip(..) => "invalid_skip",
            H::UnknownVariable(..) => "unknown_variable",
            H::NotVariable(..) => "not_variable",
            H::TypeMismatch(..) => "type_mismatch",
            H::UnreachableConditionalBranches(..) => "unreachable_conditional_branches",
            H::RefutablePattern(..) => "refutable_pattern",
        }
    }

    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        use Header as H;
        match self {
            H::Internal(msg)
                => msg.clone(),

            H::CompilerNoInput()
                => "no input file".to_string(),
            H::CompilerNoSuchPath(path)
                => format!("path '{path}' does not exist"),
            H::CompilerBadExtension(path)
                => format!("the file located at '{path}' is not a .mar source file"),
            H::CompilerIO(path, msg)
                => format!("failed to read file '{path}: {msg}"),

            H::InvalidDependencyPath()
                => "invalid dependency path in import query".to_string(),
            H::NoSuchDependency(path)
                => format!("file dependency '{path}' does not exist"),
            H::UnstagedDependency(path)
                => format!("file dependency '{path}' is unstaged"),
            H::SelfDependency(path)
                => format!("file {path} imports itself"),
            H::DependencyCycle()
                => "detected a dependency cycle".to_string(),
            H::OutsideDependency()
                => "import query leads outside of the working directory".to_string(),

            H::InvalidCharacterSequence(seq)
                => format!("invalid characters '{seq}'"),
            H::ExpectedToken(want, have)
                => format!("expected {want}, encounted {have} instead"),
            H::ExpectedExpression()
                => "expected an expression".to_string(),
            H::EmptyImport()
                => "empty import expression".to_string(),
            H::InvalidImportQuery()
                => "invalid import query syntax".to_string(),
            H::RedundantSuper()
                => "redundant use of the 'super' path".to_string(),
            H::FileReimported(path)
                => format!("file {path} is imported again"),
            H::InvalidInteger()
                => "invalid integer literal".to_string(),
            H::InvalidFloat()
                => "invalid float literal".to_string(),
            H::InvalidExpression()
                => "invalid expression syntax".to_string(),
            H::InvalidPattern()
                => "invalid pattern syntax".to_string(),
            H::InvalidSignature()
                => "invalid signature syntax".to_string(),
            H::InvalidLabel()
                => "invalid label syntax".to_string(),
            H::InvalidBreak(None)
                => "invalid break".to_string(),
            H::InvalidBreak(Some(name))
                => format!("invalid break to label '{name}'"),
            H::InvalidSkip(None)
                => "invalid Skip".to_string(),
            H::InvalidSkip(Some(name))
                => format!("invalid skip in label '{name}'"),
            H::UnknownVariable(name)
                => format!("unknown variable '{name}' in the current scope"),
            H::NotVariable(name)
                => format!("identifier '{name}' does not refer to a variable in the current scope"),
            H::TypeMismatch(left, right)
                => format!("type mismatch between {left} and {right}"),
            H::UnreachableConditionalBranches(1)
                => "unreachable conditional branch".to_string(),
            H::UnreachableConditionalBranches(_)
                => "unreachable conditional branches".to_string(),
            H::RefutablePattern()
                => "refutable pattern".to_string(),
        }
    }
}

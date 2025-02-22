use crate::com::{
    ir::{ConstraintString, TypeString},
    Token,
};

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
    ExpectedTypeAnnotation(Token),
    EmptyImport(),
    InvalidImportQuery(),
    RedundantSuper(),
    FileReimported(String),
    InvalidInteger(),
    InvalidFloat(),
    InvalidExpression(),
    InvalidAccessor(),
    InvalidType(),
    InvalidTypeArg(),
    InvalidPattern(),
    InvalidSignature(),
    InvalidTypeSignature(),
    InvalidTypeAnnotation(),
    InvalidLabel(),
    InvalidField(),
    InvalidBreak(Option<String>),
    InvalidSkip(Option<String>),
    UnallowedSignatureName(),
    UnknownVariable(String),
    UnknownBinding(String),
    UnknownType(String),
    UnknownVariant(String, String),
    UnknownClassItem(String, String),
    NotVariable(String),
    NotType(String),
    TypeMismatch(TypeString, TypeString),
    UnreachableConditionalBranches(usize),
    RefutablePattern(),
    UnionNoArgs(String),
    UnionVariantNoArgs(String),
    IncompleteType(),
    IncompleteVariant(String),
    IncorrectVariantArgs(String),
    IncorrectVariantArgCount(String, usize, usize),
    RecordNoArgs(String),
    UnionArgMismatch(String),
    RecordArgMismatch(String),
    NoAdmissibleRecords(),
    AmbiguousRecord(),
    UninitializedFields(String),
    UnmatchedFields(String),
    RequiredFieldValue(),
    ClassNoArgs(String),
    UninstantiatedItems(String),
    UnsatisfiedContraint(ConstraintString),
    AmbiguousConstraintSolution(ConstraintString),
    DisallowedPub(),
    ExpressionAlias(),
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
            H::ExpectedTypeAnnotation(..) => "expected_type_annotation",
            H::EmptyImport(..) => "empty_import",
            H::InvalidImportQuery(..) => "invalid_import_query",
            H::RedundantSuper(..) => "redundant_super",
            H::FileReimported(..) => "file_reimported",
            H::InvalidInteger(..) => "invalid_integer",
            H::InvalidFloat(..) => "invalid_float",
            H::InvalidExpression(..) => "invalid_expression",
            H::InvalidAccessor(..) => "invalid_accessor",
            H::InvalidType(..) => "invalid_type",
            H::InvalidTypeArg(..) => "invalid_type_arg",
            H::InvalidPattern(..) => "invalid_pattern",
            H::InvalidSignature(..) => "invalid_signature",
            H::InvalidTypeSignature(..) => "invalid_type_signature",
            H::InvalidTypeAnnotation(..) => "invalid_type_annotation",
            H::InvalidLabel(..) => "invalid_label",
            H::InvalidField(..) => "invalid_field",
            H::InvalidBreak(..) => "invalid_break",
            H::InvalidSkip(..) => "invalid_skip",
            H::UnallowedSignatureName(..) => "unallowed_signature_name",
            H::UnknownVariable(..) => "unknown_variable",
            H::UnknownBinding(..) => "unknown_binding",
            H::UnknownType(..) => "unknown_type",
            H::UnknownVariant(..) => "unknown_variant",
            H::UnknownClassItem(..) => "unknown_class_item",
            H::NotVariable(..) => "not_variable",
            H::NotType(..) => "not_type",
            H::TypeMismatch(..) => "type_mismatch",
            H::UnreachableConditionalBranches(..) => "unreachable_conditional_branches",
            H::RefutablePattern(..) => "refutable_pattern",
            H::UnionNoArgs(..) => "union_no_args",
            H::UnionVariantNoArgs(..) => "union_variant_no_args",
            H::IncompleteType(..) => "incomplete_type",
            H::IncompleteVariant(_) => "incomplete_variant",
            H::IncorrectVariantArgs(..) => "incorrect_variant_args",
            H::IncorrectVariantArgCount(..) => "incorrect_variant_arg_count",
            H::RecordNoArgs(..) => "record_no_args",
            H::UnionArgMismatch(..) => "union_arg_mismatch",
            H::RecordArgMismatch(..) => "record_arg_mismatch",
            H::NoAdmissibleRecords(..) => "no_admissible_records",
            H::AmbiguousRecord(..) => "ambiguous_record",
            H::UninitializedFields(..) => "uninitialized_fields",
            H::UnmatchedFields(..) => "unmatched_fields",
            H::RequiredFieldValue(..) => "required_fields_value",
            H::ClassNoArgs(..) => "class_no_args",
            H::UninstantiatedItems(..) => "uninstantiated_items",
            H::UnsatisfiedContraint(..) => "unsatisfied_contraint",
            H::AmbiguousConstraintSolution(..) => "ambiguous_constraint_solution",
            H::DisallowedPub(..) => "disallowed_pub",
            H::ExpressionAlias(..) => "expression_alias",
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
                => format!("expected {want}, encountered {have} instead"),
            H::ExpectedExpression()
                => "expected an expression".to_string(),
            H::ExpectedTypeAnnotation(have)
                => format!("expected a type annotation starting with {}, or a function return type annotation with {}, encountered {have} instead", Token::Colon, Token::Maps),
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
            H::InvalidAccessor()
                => "invalid accessor expression syntax".to_string(),
            H::InvalidType()
                => "invalid type syntax".to_string(),
            H::InvalidTypeArg()
                => "invalid type argument syntax".to_string(),
            H::InvalidPattern()
                => "invalid pattern syntax".to_string(),
            H::InvalidSignature()
                => "invalid signature syntax".to_string(),
            H::InvalidTypeSignature()
                => "invalid type signature syntax".to_string(),
            H::InvalidTypeAnnotation()
                => "invalid type annotation syntax".to_string(),
            H::InvalidLabel()
                => "invalid label syntax".to_string(),
            H::InvalidField()
                => "invalid record field syntax".to_string(),
            H::InvalidBreak(None)
                => "invalid break".to_string(),
            H::InvalidBreak(Some(name))
                => format!("invalid break to label '{name}'"),
            H::InvalidSkip(None)
                => "invalid Skip".to_string(),
            H::UnallowedSignatureName()
                => "unallowed signature name".to_string(),
            H::InvalidSkip(Some(name))
                => format!("invalid skip in label '{name}'"),
            H::UnknownVariable(name)
                => format!("unknown variable '{name}' in the current scope"),
            H::UnknownBinding(name)
                => format!("unknown binding '{name}' in the current scope"),
            H::UnknownType(name)
                => format!("unknown type '{name}' in the current scope"),
            H::UnknownVariant(name, union_name)
                => format!("unknown variant '{name}' in union type '{union_name}'"),
            H::UnknownClassItem(name, class_name)
                => format!("unknown item '{name}' in class '{class_name}'"),
            H::NotVariable(name)
                => format!("identifier '{name}' does not refer to a variable in the current scope"),
            H::NotType(name)
                => format!("identifier '{name}' does not refer to a type in the current scope"),
            H::TypeMismatch(left, right)
                => format!("type mismatch between {left} and {right}"),
            H::UnreachableConditionalBranches(1)
                => "unreachable conditional branch".to_string(),
            H::UnreachableConditionalBranches(_)
                => "unreachable conditional branches".to_string(),
            H::RefutablePattern()
                => "refutable pattern".to_string(),
            H::UnionNoArgs(name)
                => format!("non-constant union type '{name}' has no arguments"),
            H::UnionVariantNoArgs(name)
                => format!("non-constant union variant '{name}' has no arguments"),
            H::IncompleteType()
                => "incomplete type expression".to_string(),
            H::IncompleteVariant(name)
                => format!("variant pattern '{name}' is incomplete"),
            H::IncorrectVariantArgs(name)
                => format!("variant '{name}' takes no arguments"),
            H::IncorrectVariantArgCount(name, 1, have)
                => format!("variant '{name}' takes a single argument, but received {have}"),
            H::IncorrectVariantArgCount(name, want, 1)
                => format!("variant '{name}' takes {want} arguments, but received only one"),
            H::IncorrectVariantArgCount(name, want, have)
                => format!("variant '{name}' takes {want} arguments, but received {have}"),
            H::RecordNoArgs(name)
                => format!("non-constant record type '{name}' has no arguments"),
            H::UnionArgMismatch(name)
                => format!("invalid number of arguments provided into union type '{name}'"),
            H::RecordArgMismatch(name)
                => format!("invalid number of arguments provided into record type '{name}'"),
            H::NoAdmissibleRecords()
                => "no admissible record type in the current scope".to_string(),
            H::AmbiguousRecord()
                => "ambiguous record type for given fields".to_string(),
            H::UninitializedFields(record)
                => format!("record type '{record}' is not fully initialized"),
            H::UnmatchedFields(record)
                => format!("record type '{record}' is not fully matched"),
            H::RequiredFieldValue()
                => "record field requires a value to be initialized".to_string(),
            H::ClassNoArgs(name)
                => format!("class '{name}' has no type arguments"),
            H::UninstantiatedItems(class_name)
                => format!("instantiation of class '{class_name}' is incomplete"),
            H::UnsatisfiedContraint(constraint)
                => format!("unsatisfied constraint [{constraint}]"),
            H::AmbiguousConstraintSolution(constraint)
                => format!("ambiguous solution for constraint [{constraint}]"),
            H::DisallowedPub()
                => "disallowed usage of the 'pub' access modifier".to_string(),
            H::ExpressionAlias()
                => "disallowed usage of 'alias' for an expression".to_string(),
        }
    }
}

use crate::com::ir::TypeString;

#[derive(Clone)]
pub enum Label {
    Empty,
    ExpectedImportQuery,
    ImportedInFile(String),
    ImportedInItself(String),
    TrailingSuper,
    RedundantImportPath,
    FirstImportHere(String),
    Type(TypeString),
    ArrayItemTypes,
    ReturnValueTypes(Option<String>),
    ReturnedFromBreak(Option<String>),
    NoBreakpointFound(Option<String>),
    NoSkippointFound(Option<String>),
    UnskippableBlock(Option<String>),
    ConditionBoolType,
    ConditionalReturnValues,
    NonExhaustiveConditionalUnit,
    ExhaustiveConditionalBranches(usize),
    UnreachableConditionalBranches(usize),
    VariableDefinition(String),
    NamelessSignature,
    LetBindingPattern,
    FunctionArgPattern,
    WithinUnionDefinition(String),
    UnionTypeArgCount(String, usize),
    UnionTypeNoArgs(String),
    UnionDefinition(String),
    VariantArgCount(String, usize),
    VariantDefinition(String),
    NotAnExpression,
    NotAPattern,
    NotAType,
    WantFunctionType(TypeString),
    WithinRecordDefinition(String),
    RecordTypeArgCount(String, usize),
    RecordTypeNoArgs(String),
    RecordDefinition(String),
    NoAdmissibleRecord(usize),
    MissingFields(Box<[String]>, String),
}

impl Label {
    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        use Label as L;
        match self {
            L::Empty
                => "".to_string(),
            L::ExpectedImportQuery
                => "expected one or more import queries".to_string(),
            L::ImportedInFile(path)
                => format!("imported in file {path}"),
            L::ImportedInItself(path)
                => format!("{path} is imported it itself"),
            L::TrailingSuper
                => "import queries cannot end with 'super'".to_string(),
            L::RedundantImportPath
                => "this part of the import query is redundant".to_string(),
            L::FirstImportHere(path)
                => format!("first import of {path} here"),
            L::Type(ty)
                => ty.to_string(),
            L::ArrayItemTypes
                => "all items in an array must be of the same type".to_string(),
            L::ReturnValueTypes(Some(name))
                => format!("all values returned by label '{name}' must be of the same type"),
            L::ReturnValueTypes(None)
                => "all values returned at this point must be of the same type".to_string(),
            L::ReturnedFromBreak(None)
                => "this value is returned from this break expression".to_string(),
            L::ReturnedFromBreak(Some(name))
                => format!("this value is returned from this break out of label '{name}'"),
            L::NoBreakpointFound(None)
                => "there are no control flow structures to break out of in this scope".to_string(),
            L::NoBreakpointFound(Some(name))
                => format!("there are no control flow structures with label '{name}' to break out of in this scope"),
            L::NoSkippointFound(None)
                => "there are no control flow structures inside which to skip".to_string(),
            L::NoSkippointFound(Some(name))
                => format!("there are no control flow structures with label '{name}' inside which to skip"),
            L::UnskippableBlock(None)
                => "cannot skip inside any current control flow structure, as none of them are loops".to_string(),
            L::UnskippableBlock(Some(name))
                => format!("cannot skip inside label '{name}', as it is not a loop"),
            L::ConditionBoolType
                => "condition expression must be a boolean".to_string(),
            L::ConditionalReturnValues
                => "all values returned by this conditional expression must be of the same type".to_string(),
            L::NonExhaustiveConditionalUnit
                => "non-exhaustive conditional expression returns unit".to_string(),
            L::ExhaustiveConditionalBranches(1)
                => "the previous conditional branch is exhaustive".to_string(),
            L::ExhaustiveConditionalBranches(_)
                => "the previous conditional branches are exhaustive".to_string(),
            L::UnreachableConditionalBranches(1)
                => "this conditional branch is never reached".to_string(),
            L::UnreachableConditionalBranches(_)
                => "these conditional branches are never reached".to_string(),
            L::VariableDefinition(name)
                => format!("variable '{name}' is defined here"),
            L::NamelessSignature
                => "function signature has no name".to_string(),
            L::LetBindingPattern
                => "let-binding patterns must be irrefutable".to_string(),
            L::FunctionArgPattern
                => "function argument patterns must be irrefutable".to_string(),
            L::WithinUnionDefinition(name)
                => format!("within the definition of union type '{name}'"),
            L::UnionTypeArgCount(name, 1)
                => format!("union type '{name}' takes in a single argument"),
            L::UnionTypeArgCount(name, count)
                => format!("union type '{name}' takes in {count} arguments"),
            L::UnionTypeNoArgs(name)
                => format!("union type '{name}' does not take in any argument"),
            L::UnionDefinition(name)
                => format!("union type '{name}' is defined here"),
            L::VariantArgCount(name, 1)
                => format!("variant '{name}' takes in a single argument"),
            L::VariantArgCount(name, count)
                => format!("variant '{name}' takes in {count} arguments"),
            L::VariantDefinition(name)
                => format!("variant '{name}' is defined here"),
            L::NotAnExpression
                => "this is valid syntax but does not represent a value".to_string(),
            L::NotAPattern
                => "this is valid syntax but does not represent a pattern".to_string(),
            L::NotAType
                => "this is valid syntax but does not represent a type".to_string(),
            L::WantFunctionType(ty)
                => format!("this expression is expected to be a function of type {ty}"),
            L::WithinRecordDefinition(name)
                => format!("within the definition of record type '{name}'"),
            L::RecordTypeArgCount(name, 1)
                => format!("record type '{name}' takes in a single argument"),
            L::RecordTypeArgCount(name, count)
                => format!("record type '{name}' takes in {count} arguments"), 
            L::RecordTypeNoArgs(name)
                => format!("record type '{name}' does not take in any argument"),
            L::RecordDefinition(name)
                => format!("record type '{name}' is defined here"),
            L::NoAdmissibleRecord(0)
                => "no record type in this scope contains zero fields".to_string(),
            L::NoAdmissibleRecord(1)
                => "no record type in this scope contains this field".to_string(),
            L::NoAdmissibleRecord(_)
                => "no record type in this scope contains all these fields at the same time".to_string(),
            L::MissingFields(fields, record) if fields.len() == 1
                => format!("record type '{record}' is missing field: '{}'", &fields[0]),
            L::MissingFields(fields, record) 
                => format!("record type '{record}' is missing fields: {}", fields.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(", ")),
        }
    }
}

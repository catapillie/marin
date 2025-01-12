pub enum Note {
    ConsiderStage(String),
    CyclicDependencies(Vec<String>),
    RecordSyntax,
    RecordFieldSyntax,
    UnionSyntax,
    UnionVariantSyntax,
    UseSimpleRecordSyntax(String),
    UseSimpleUnionSyntax(String),
    UseConstantUnionSyntax(String),
}

impl Note {
    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        use Note as N;
        match self {
            N::ConsiderStage(path)
                => format!("consider staging {path} in the compile command"),
            N::CyclicDependencies(deps)
                => format!("files involved in the cycle\n  {}", deps.join("\n  ")),
            N::RecordSyntax
                => "a record signature must have a name, and may optionally be followed by one or more arguments within parentheses".to_string(),
            N::RecordFieldSyntax
                => "a record field should be an identifier".to_string(),
            N::UnionSyntax
                => "a union signature must have a name, and may optionally be followed by one or more arguments within parentheses".to_string(),
            N::UnionVariantSyntax
                => "a union variant must have a name, and may optionally be followed by one or more arguments within parentheses".to_string(),
            N::UseSimpleRecordSyntax(name)
                => format!("consider removing the parentheses to make '{name}' a record type with no type arguments"),
            N::UseSimpleUnionSyntax(name)
                => format!("consider removing the parentheses to make '{name}' a union type with no type arguments"),
            N::UseConstantUnionSyntax(name)
                => format!("consider removing the parentheses to make '{name}' a constant union variant"),
        }
    }
}

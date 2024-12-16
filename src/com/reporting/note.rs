pub enum Note {
    ConsiderStage(String),
    CyclicDependencies(Vec<String>),
    UnionSyntax,
    VariantSyntax,
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
            N::UnionSyntax
                => "a union signature must have a name, and may optionally be followed by one or more arguments within parentheses".to_string(),
            N::VariantSyntax
                => "a union variant must have a name, and may optionally be followed by one or more arguments within parentheses".to_string(),
            N::UseSimpleUnionSyntax(name)
                => format!("consider removing the parentheses to make '{name}' a union type with no type arguments"),
            N::UseConstantUnionSyntax(name)
                => format!("consider removing the parentheses to make '{name}' a constant union variant"),
        }
    }
}

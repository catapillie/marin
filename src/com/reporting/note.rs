pub enum Note {
    ConsiderStage(String),
    CyclicDependencies(Vec<String>),
    UnionVariantSyntax,
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
            N::UnionVariantSyntax
                => "a union variant should contain a name, and optionally be followed by one or more arguments within parentheses".to_string(),
        }
    }
}

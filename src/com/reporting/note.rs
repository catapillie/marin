pub enum Note {
    ConsiderStage(String),
    CyclicDependencies(Vec<String>),
}

impl Note {
    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        use Note as N; 
        match self {
            N::ConsiderStage(path)
                => format!("consider staging the file dependency: {path}"),
            N::CyclicDependencies(deps)
                => format!("files involved in the cycle\n  {}", deps.join("\n  ")),
        }
    }
}

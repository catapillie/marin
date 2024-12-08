use crate::com::ir::TypeString;

pub enum Label {
    Empty,
    ExpectedImportQuery,
    ImportedInFile(String),
    ImportedInItself(String),
    TrailingSuper,
    RedundantImportPath,
    FirstImportHere(String),
    Type(TypeString),
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
        }
    }
}

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
    ArrayItemTypes,
    ReturnValueTypes(Option<String>),
    ReturnedFromBreak(Option<String>),
    NoBreakpointFound(Option<String>),
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
            L::NoBreakpointFound(None)
                => "there are no control flow structures to break out of in this scope".to_string(),
            L::ReturnedFromBreak(None)
                => "this value is returned from this break expression".to_string(),
            L::ReturnedFromBreak(Some(name))
                => format!("this value is returned from this break out of the '{name}' label"),
            L::NoBreakpointFound(Some(name))
                => format!("there are no control flow structures with label '{name}' to break out of in this scope"),
        }
    }
}

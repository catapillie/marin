pub enum Label {
    Empty,
}

impl Label {
    #[rustfmt::skip]
    pub fn msg(&self) -> String {
        match self {
            Label::Empty => "".to_string(),
        }
    }
}

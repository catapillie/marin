use crate::com::{
    loc::Loc,
    reporting::{Label, Report},
};

pub enum Provenance {
    ArrayItems(Loc),
}

impl Provenance {
    pub fn apply(&self, report: Report) -> Report {
        use Provenance as Pr;
        match self {
            Pr::ArrayItems(loc) => report.with_secondary_label(Label::ArrayItemTypes, *loc),
        }
    }
}

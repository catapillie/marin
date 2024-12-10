use crate::com::{
    loc::Loc,
    reporting::{Label, Report},
};

pub enum Provenance {
    ArrayItems(Loc),
    LabelValues(Loc, Option<String>),
    ConditionalBoolType(Loc),
    ConditionalReturnValues(Loc),
}

impl Provenance {
    pub fn apply(&self, report: Report) -> Report {
        use Provenance as Pr;
        match self {
            Pr::ArrayItems(loc) => report.with_secondary_label(Label::ArrayItemTypes, *loc),
            Pr::LabelValues(loc, name) => {
                report.with_secondary_label(Label::ReturnValueTypes(name.clone()), *loc)
            }
            Pr::ConditionalBoolType(loc) => {
                report.with_secondary_label(Label::ConditionBoolType, *loc)
            }
            Pr::ConditionalReturnValues(loc) => {
                report.with_secondary_label(Label::ConditionalReturnValues, *loc)
            }
        }
    }
}

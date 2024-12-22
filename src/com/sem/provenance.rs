use crate::com::{
    ir::TypeString,
    loc::Loc,
    reporting::{Label, Report},
};

pub enum Provenance {
    ArrayItems(Loc),
    LabelValues(Loc, Option<String>),
    ConditionalBoolType(Loc),
    ConditionalReturnValues(Loc),
    FunctionCall(TypeString, Loc),
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
            Pr::FunctionCall(ty, loc) => {
                report.with_secondary_label(Label::WantFunctionType(ty.clone()), *loc)
            }
        }
    }
}

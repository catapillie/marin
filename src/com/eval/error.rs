#[derive(Debug)]
pub enum Error {
    Missing,
    InvalidState,
    NonBooleanCondition,
    PatternMismatch,
    UnknownVariable,
    InvalidFunction,
    InvalidArgCount,
}

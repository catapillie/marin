#[derive(Debug)]
pub enum Error {
    Missing,
    UnknownVariable(String),
}

use super::Pattern;

#[derive(Debug, Clone)]
pub enum Signature {
    Missing,
    Args(Box<[Pattern]>, Box<Signature>),
    Done,
}

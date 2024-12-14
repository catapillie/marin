use super::Pattern;

#[derive(Debug)]
pub enum Signature {
    Missing,
    Args(Box<[Pattern]>, Box<Signature>),
    Done,
}

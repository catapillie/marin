use super::Pattern;

#[derive(Debug, Clone)]
pub enum Signature {
    Missing,
    Args {
        args: Box<[Pattern]>,
        next: Box<Signature>,
    },
    Done,
}

use std::{io, result, string};

pub type Result<T> = result::Result<T, Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    MagicMismatch,
    IllegalOpcode,
    IllegalValue,
    IO(io::Error),
    Utf8(string::FromUtf8Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(error: string::FromUtf8Error) -> Self {
        Error::Utf8(error)
    }
}

use std::fmt::{Display, Formatter};
use std::io::Error as IOError;
use std::string::FromUtf8Error;

use crate::io::var::VarInt;

#[derive(Debug)]
pub enum Error {
    Eof,
    TooBig,
    Other,
    InvalidPacketId(VarInt),
    Utf8(FromUtf8Error),
    IO(IOError),
    Timeout,
    LegacyPing,
}

impl From<IOError> for Error {
    fn from(value: IOError) -> Self {
        Error::IO(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::Utf8(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

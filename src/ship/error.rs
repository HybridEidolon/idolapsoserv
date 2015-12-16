use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::result;
use std::io;

use std::error::Error as ErrorTrait;

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    IoError,
    DbError,
    CredentialError,
    UnexpectedMsg,
    Other
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    sub_error: Box<error::Error + Send + Sync>
}

impl ErrorKind {
    pub fn to_str(self) -> &'static str {
        use self::ErrorKind::*;
        match self {
            IoError => "An internal IO error occurred.",
            DbError => "An internal database error occurred.",
            CredentialError => "Credential error.",
            UnexpectedMsg => "An unexpected message was received.",
            Other => "Other, unspecified error. The developers should be more specific.",
            //u => "Unknown error"
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        self.kind.to_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(self.sub_error.as_ref() as &error::Error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> result::Result<(), fmt::Error> {
        match self.cause() {
            Some(e) => write!(fmt, "ShipError: {}.\nCaused by: {}", self.description(), e),
            None => write!(fmt, "ShipError: {}.", self.description())
        }
    }
}

impl Error {
    pub fn new<E: Into<Box<ErrorTrait + Send + Sync>>>(kind: ErrorKind, e: E) -> Error {
        Error {
            kind: kind,
            sub_error: e.into()
        }
    }
}

impl From<io::Error> for Error {
    fn from(t: io::Error) -> Error {
        Error::new(ErrorKind::IoError, t)
    }
}

pub type Result<T> = result::Result<T, Error>;

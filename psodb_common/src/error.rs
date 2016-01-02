//! Main error type for database functionality.

use std::error;
use std::io;

use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    BackendError(Option<Box<error::Error>>),
    IoError(io::Error),
    Other(String, Option<Box<error::Error>>)
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use std::error::Error;
        write!(f, "database error: {}\ncaused by: {:?}",
            self.description(),
            self.cause())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match self {
            &BackendError(Some(ref e)) => e.description(),
            &BackendError(None) => "",
            &IoError(ref e) => e.description(),
            &Other(ref s, _) => &s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match self {
            &BackendError(Some(ref o)) => Some(o.as_ref()),
            &IoError(ref e) => Some(e),
            &Other(_, Some(ref o)) => Some(o.as_ref()),
            _ => None
        }
    }
}

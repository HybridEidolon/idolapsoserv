//! Implements an abstraction over Phantasy Star Online's many message namespaces.
//! Intended for use with the IDOLA server project.

extern crate byteorder;
extern crate encoding;
extern crate typenum;
extern crate psocrypto;

#[macro_use] extern crate log;

pub mod staticvec;

#[macro_use] pub mod serial;

pub use ::serial::*;
pub mod util;
pub mod common;

pub mod patch;
pub mod bb;

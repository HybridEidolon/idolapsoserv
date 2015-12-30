//! Implements an abstraction over Phantasy Star Online's many message namespaces.
//! Intended for use with the IDOLA server project.

extern crate byteorder;
extern crate encoding;
extern crate typenum;
#[macro_use] extern crate psoserial;
#[macro_use] extern crate log;
extern crate staticvec;

extern crate psomsg_common;
extern crate psomsg_patch;
extern crate psomsg_bb;

pub use psoserial::Serial;

pub mod util;
pub mod common;

pub mod patch;
pub mod bb;

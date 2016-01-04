//! The login service, forwards to a random ship which provides
//! ship selection information.

pub mod bb;
pub mod paramfiles;

pub use self::bb::BbLoginService;

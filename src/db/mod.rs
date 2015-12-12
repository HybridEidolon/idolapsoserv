//! Database abstraction for managing persistent data, such as accounts and characters.

pub mod error;

pub mod account;

pub mod sqlite;

pub use self::error::Error;
pub use self::account::Account;

use std::result;

/// Wrapper around the standard result that yields the database error type for Err.
pub type Result<T> = result::Result<T, Error>;

/// A backend implementation for the database. When receiving a trait object on this trait, the
/// implementing type should already have initialized its resources so the methods would succeed
/// under normal conditions.
pub trait Backend {
    /// Retrieve an account by its ID.
    fn get_account_by_id(&self, id: u32) -> Result<Option<Account>>;

    /// Retrieve an account by its username.
    fn get_account_by_username<U: Into<String>>(&self, username: U) -> Result<Option<Account>>;

    /// Insert or update an account into the database, based on its internal ID value.
    fn put_account(&self, account: &mut Account) -> Result<()>;

    /// Reset or invalidate the passwords of every account.
    fn reset_account_passwords(&self) -> Result<()>;
}

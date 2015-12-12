//! Sqlite3 database backend.

use std::path::Path;

use super::Result;
use super::Backend;
use super::error::Error;

use super::account::Account;

// yo someone needs to tell the rusqlite people this naming scheme is bad
use sqlite::Connection;

pub struct Sqlite {
    conn: Connection
}

impl Sqlite {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Sqlite> {
        Ok(Sqlite {
            conn: match Connection::open(path) {
                Ok(c) => c,
                Err(e) => return Err(
                    Error::BackendError(
                        "Couldn't create new Sqlite backend".to_string(),
                        Some(Box::new(e))))
            }
        })
    }
}

impl Backend for Sqlite {
    fn get_account_by_id(&self, id: u32) -> Result<Option<Account>> {
        unimplemented!()
    }

    fn get_account_by_username<U: Into<String>>(&self, username: U) -> Result<Option<Account>> {
        unimplemented!()
    }

    fn put_account(&self, account: &mut Account) -> Result<()> {
        unimplemented!()
    }

    fn reset_account_passwords(&self) -> Result<()> {
        unimplemented!()
    }
}

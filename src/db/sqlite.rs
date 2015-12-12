//! Sqlite3 database backend.

use std::path::Path;

use super::Result;
use super::Backend;
use super::error::Error;

use super::account::Account;

// yo someone needs to tell the rusqlite people this naming scheme is bad
use sqlite::{Connection, Value};

pub struct Sqlite {
    conn: Connection
}

impl Sqlite {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Sqlite> {
        Ok(Sqlite {
            conn: match Connection::open(path) {
                Ok(c) => c,
                Err(e) => return Err(
                    Error::BackendError(Some(Box::new(e))))
            }
        })
    }
}

macro_rules! try_db {
    ($e:expr) => {
        match $e {
            Ok(s) => s,
            Err(e) => return Err(Error::BackendError(Some(Box::new(e))))
        }
    }
}

impl Backend for Sqlite {
    fn get_account_by_id(&self, id: u32) -> Result<Option<Account>> {
        let prep = try_db!(self.conn.prepare(
            "SELECT username,password_hash,password_invalidated,banned FROM accounts WHERE id=? LIMIT 1"));
        let mut cursor = prep.cursor();
        try_db!(cursor.bind(&[Value::Integer(id as i64)]));
        match cursor.next() {
            Ok(Some(r)) => {
                Ok(Some(Account {
                    id: Some(id),
                    username: r[0].as_string().unwrap_or("").to_owned(),
                    password_hash: r[1].as_string().unwrap_or("").to_owned(),
                    password_invalidated: r[2].as_integer().map(|i| i != 0).unwrap_or(true),
                    banned: r[3].as_integer().map(|i| i != 0).unwrap_or(false)
                }))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(Error::BackendError(Some(Box::new(e))))
        }
    }

    fn get_account_by_username<U: Into<String>>(&self, username: U) -> Result<Option<Account>> {
        let username = username.into();
        let prep = try_db!(self.conn.prepare(
            "SELECT id,password_hash,password_invalidated,banned FROM accounts WHERE username=? LIMIT 1"
        ));
        let mut cursor = prep.cursor();
        try_db!(cursor.bind(&[Value::String(username.clone())]));
        match cursor.next() {
            Ok(Some(row)) => {
                Ok(Some(Account {
                    id: row[0].as_integer().map(|i| i as u32),
                    username: username.clone(),
                    password_hash: row[1].as_string().unwrap_or("").to_owned(),
                    password_invalidated: row[2].as_integer().map(|i| i != 0).unwrap_or(true),
                    banned: row[3].as_integer().map(|i| i != 0).unwrap_or(false)
                }))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(Error::BackendError(Some(Box::new(e))))
        }
    }

    fn put_account(&self, account: &mut Account) -> Result<()> {
        match account.id {
            Some(id) => {
                let mut stmt = try_db!(self.conn.prepare("UPDATE accounts (username,password_hash,password_invalidated,banned) WHERE id=? VALUES (?,?,?,?)")).cursor();
                try_db!(stmt.bind(&[
                    Value::Integer(id as i64),
                    Value::String(account.username.clone()),
                    Value::String(account.password_hash.clone()),
                    Value::Integer(if account.password_invalidated {1} else {0}),
                    Value::Integer(if account.banned {1} else {0})]));
                try_db!(stmt.next());
                Ok(())
            },
            None => {
                let mut stmt = try_db!(self.conn.prepare("INSERT INTO accounts (username,password_hash,password_invalidated,banned) VALUES (?,?,?,?)")).cursor();
                try_db!(stmt.bind(&[
                    Value::String(account.username.clone()),
                    Value::String(account.password_hash.clone()),
                    Value::Integer(if account.password_invalidated {1} else {0}),
                    Value::Integer(if account.banned {1} else {0})]));
                try_db!(stmt.next());
                Ok(())
            }
        }
    }

    fn reset_account_passwords(&self) -> Result<()> {
        let mut stmt = try_db!(self.conn.prepare("UPDATE accounts (password_invalidated) VALUES (1)")).cursor();
        try_db!(stmt.next());
        Ok(())
    }
}

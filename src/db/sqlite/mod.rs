//! Sqlite3 database backend.

use std::path::Path;

use super::Result;
use super::Backend;
use super::error::Error;

use super::account::Account;

mod schema;
use self::schema::SCHEMA;

#[cfg(test)] mod test;

// yo someone needs to tell the rusqlite people this naming scheme is bad
use rusqlite::SqliteConnection as Connection;

macro_rules! try_db {
    ($e:expr) => {
        match $e {
            Ok(s) => s,
            Err(e) => return Err(Error::BackendError(Some(Box::new(e))))
        }
    }
}

pub struct Sqlite {
    conn: Connection
}

impl Sqlite {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Sqlite> {
        let conn = try_db!(Connection::open(path));
        try_db!(Sqlite::initialize_tables(&conn));

        Ok(Sqlite {
            conn: conn
        })
    }

    fn initialize_tables(c: &Connection) -> Result<()> {
        try_db!(c.execute_batch(SCHEMA));
        Ok(())
    }
}


// TEMPORARY -- REMOVE THESE ON RUSQLITE UPDATE
#[inline(always)] fn b2i(a: bool) -> i64 { match a { true => 1, false => 0 }}
#[inline(always)] fn i2b(a: i64) -> bool { match a { 0 => false, _ => true }}

impl Backend for Sqlite {
    fn get_account_by_id(&self, id: u32) -> Result<Option<Account>> {
        let id = id as i64;
        let mut stmt = try_db!(self.conn.prepare(
            "SELECT username,password_hash,password_invalidated,banned FROM accounts WHERE id=? LIMIT 1"));

        let mut results = try_db!(stmt.query_map(&[&id], |row| {
            Account {
                id: Some(id as u32),
                username: row.get(0),
                password_hash: row.get(1),
                password_invalidated: i2b(row.get(2)),
                banned: i2b(row.get(3))
                // TODO when rusqlite updates, make these ::<bool>. 0.5.0 doesn't impl bool
            }
        }));
        match results.next() {
            Some(Ok(a)) => Ok(Some(a)),
            Some(Err(e)) => Err(Error::BackendError(Some(Box::new(e)))),
            None => Ok(None)
        }
    }

    fn get_account_by_username<U: Into<String>>(&self, username: U) -> Result<Option<Account>> {
        let u = username.into();
        let uc = u.clone();
        let mut stmt = try_db!(self.conn.prepare(
            "SELECT id,password_hash,password_invalidated,banned FROM accounts WHERE username=? LIMIT 1"
        ));

        let mut results = try_db!(stmt.query_map(&[&u], |row| {
            Account {
                id: Some(row.get::<i64>(0) as u32),
                username: uc.clone(),
                password_hash: row.get(1),
                password_invalidated: i2b(row.get(2)),
                banned: i2b(row.get(3))
            }
        }));
        match results.next() {
            Some(Ok(a)) => Ok(Some(a)),
            Some(Err(e)) => Err(Error::BackendError(Some(Box::new(e)))),
            None => Ok(None)
        }
    }

    fn put_account(&self, account: &mut Account) -> Result<()> {
        match account.id {
            Some(id) => {
                let id = id as i64;
                let mut stmt = try_db!(self.conn.prepare("UPDATE accounts (username,password_hash,password_invalidated,banned) WHERE id=? VALUES (?,?,?,?)"));
                try_db!(stmt.execute(&[&id, &account.username, &account.password_hash, &b2i(account.password_invalidated), &b2i(account.banned)]));
                Ok(())
            },
            None => {
                let mut stmt = try_db!(self.conn.prepare("INSERT INTO accounts (username,password_hash,password_invalidated,banned) VALUES (?,?,?,?)"));
                try_db!(stmt.execute(&[&account.username, &account.password_hash, &b2i(account.password_invalidated), &b2i(account.banned)]));
                account.id = Some(self.conn.last_insert_rowid() as u32);
                Ok(())
            }
        }
    }

    fn reset_account_passwords(&self) -> Result<()> {
        try_db!(self.conn.execute("UPDATE accounts (password_invalidated) VALUES (1)", &[]));
        Ok(())
    }
}

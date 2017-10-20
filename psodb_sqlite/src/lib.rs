//! Sqlite3 database backend.

#[macro_use] extern crate log;
extern crate rusqlite;
extern crate psodb_common;
extern crate psodata;
extern crate psoserial;

use std::io::Cursor;

use psoserial::Serial;

use psodb_common::Result;
use psodb_common::Backend;
use psodb_common::error::Error;

use psodb_common::account::Account;
use psodb_common::account::BbAccountInfo;

use psodata::chara::{BbFullCharData, BbTeamAndKeyData, BbChar};

mod schema;
use self::schema::SCHEMA;

#[cfg(test)] mod test;

use rusqlite::Connection;
use rusqlite::types::ToSql;

macro_rules! try_db {
    ($e:expr) => {
        match $e {
            Ok(s) => s,
            Err(e) => return Err(Error::BackendError(Some(Box::new(e))))
        }
    }
}

/// A wrapper around the Sqlite implementation's connection, to implement Backend.
pub struct Sqlite {
    path: String,
    conn: Connection
}

impl Sqlite {
    /// Create a new Sqlite instance, initializing the schema of the database and applying
    /// all migrations needed to update it. The `new` argument allows you to initialize a new
    /// database up to the current schema, rather than checking and migrating the database.
    ///
    /// This is a destructive operation. Reverse migrations may destroy data from the database.
    pub fn new<T: Into<String>>(path: T, new: bool) -> Result<Sqlite> {
        let p = path.into();
        let conn = try_db!(Connection::open(&p));
        if new {
            try_db!(Sqlite::initialize_tables(&conn));
        } else {
            try_db!(Sqlite::migrate(&conn, 0));
        }

        Ok(Sqlite {
            path: p,
            conn: conn
        })
    }

    /// Initialize and update tables
    fn initialize_tables(c: &Connection) -> Result<()> {

        try_db!(c.execute_batch(SCHEMA));
        Ok(())
    }

    /// Migrate the database
    fn migrate(_: &Connection, _: i64) -> Result<()> {
        //let version = try_db!(c.query_row("SELECT version FROM version LIMIT 1"), &[], |r| r.get::<_, i64>(0));
        Ok(())
    }

    // fn reverse_migrations(c: &Connection, v: i64) -> Result<()> {
    //     Ok(())
    // }
}


// TEMPORARY -- REMOVE THESE ON RUSQLITE UPDATE
#[inline(always)] fn b2i(a: bool) -> i64 { match a { true => 1, false => 0 }}
#[inline(always)] fn i2b(a: i64) -> bool { match a { 0 => false, _ => true }}

impl Backend for Sqlite {
    fn try_clone(&mut self) -> Result<Box<Backend>> {
        let c = try_db!(Connection::open(&self.path.clone()));
        Ok(Box::new(Sqlite {
            path: self.path.clone(),
            conn: c
        }))
    }

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

    fn get_account_by_username(&self, username: &str) -> Result<Option<Account>> {
        let mut stmt = try_db!(self.conn.prepare(
            "SELECT id,password_hash,password_invalidated,banned FROM accounts WHERE username=? LIMIT 1"
        ));

        let mut results = try_db!(stmt.query_map(&[&username], |row| {
            Account {
                id: Some(row.get::<_, i64>(0) as u32),
                username: username.to_owned(),
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
                let mut stmt = try_db!(self.conn.prepare("UPDATE accounts SET (username=?,password_hash=?,password_invalidated=?,banned=?) WHERE id=?"));
                try_db!(stmt.execute(&[&account.username, &account.password_hash, &b2i(account.password_invalidated), &b2i(account.banned), &id]));
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
        try_db!(self.conn.execute("UPDATE accounts SET password_invalidated=1", &[]));
        Ok(())
    }

    fn fetch_bb_account_info(&self, account_id: u32) -> Result<Option<BbAccountInfo>> {
        let mut stmt = try_db!(self.conn.prepare(
            "SELECT id,team_id,options,key_config,joy_config,shortcuts,symbol_chats FROM bb_guildcard WHERE account_id=? LIMIT 1"
        ));

        let mut results = try_db!(stmt.query_map(&[&(account_id as i64)], |row| {
            BbAccountInfo {
                account_id: account_id,
                guildcard_num: row.get::<_, i64>(0) as u32,
                team_id: row.get::<_, i64>(1) as u32,
                options: row.get::<_, i64>(2) as u32,
                key_config: row.get::<_, Vec<u8>>(3),
                joy_config: row.get::<_, Vec<u8>>(4),
                shortcuts: row.get::<_, Vec<u8>>(5),
                symbol_chats: row.get::<_, Vec<u8>>(6)
            }
        }));

        match results.next() {
            Some(Ok(a)) => Ok(Some(a)),
            Some(Err(e)) => Err(Error::BackendError(Some(Box::new(e)))),
            None => {
                // create defaults and push them to the database
                let mut a = BbAccountInfo::new();
                a.account_id = account_id;
                match self.put_bb_account_info(&a) {
                    Ok(_) => Ok(Some(a)),
                    Err(e) => Err(e)
                }
            }
        }
    }

    fn put_bb_account_info(&self, info: &BbAccountInfo) -> Result<()> {
        let id = info.account_id as i64;
        let gcnum = info.guildcard_num as i64;
        let team = info.team_id as i64;
        let options = info.options as i64;
        let mut stmt = try_db!(self.conn.prepare("INSERT OR REPLACE INTO bb_guildcard (id,account_id,team_id,options,key_config,joy_config,shortcuts,symbol_chats) VALUES (?,?,?,?,?,?,?,?)"));
        try_db!(stmt.execute(&[&gcnum, &id, &team, &options, &info.key_config, &info.joy_config, &info.shortcuts, &info.symbol_chats]));
        Ok(())
    }

    fn fetch_bb_character(&self, account_id: u32, slot: u8) -> Result<Option<BbFullCharData>> {
        let id = account_id as i64;
        let slot = slot as i64;
        // First, fetch their BB account data.
        let acc_info = match self.fetch_bb_account_info(account_id) {
            Ok(Some(info)) => info,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e)
        };

        debug!("Account info for {} retrieved", account_id);
        let mut query = try_db!(self.conn.prepare("SELECT
            inventory,
            char_data,
            quest_data1,
            bank,
            guildcard_desc,
            autoreply,
            infoboard,
            challenge_data,
            tech_menu,
            quest_data2 FROM bb_character WHERE account_id=? AND slot=?"));
        let mut results = match query.query_map(&[&id, &slot], |row| {
            let chara: BbChar = try_db!(Serial::deserialize(&mut Cursor::new(row.get::<_, Vec<u8>>(1))));
            let key_config = BbTeamAndKeyData {
                unk: vec![0; 276],
                key_config: acc_info.key_config.clone(),
                joy_config: acc_info.joy_config.clone(),
                guildcard: 0, // TODO no teams yet
                team_id: 0,
                team_info: (0, 0),
                team_priv: 0,
                team_name: "".to_string(),
                team_flag: vec![0; 2048],
                team_rewards: 0
            };
            Ok(BbFullCharData {
                inv: try_db!(Serial::deserialize(&mut Cursor::new(row.get::<_, Vec<u8>>(0)))),
                chara: chara.clone(),
                unk: vec![0; 0x0010],
                option_flags: acc_info.options,
                quest_data1: row.get::<_, Vec<u8>>(2),
                bank: try_db!(Serial::deserialize(&mut Cursor::new(row.get::<_, Vec<u8>>(3)))),
                guildcard: acc_info.guildcard_num,
                name: chara.name.clone(),
                team_name: "".to_string(), // TODO no teams yet
                guildcard_desc: row.get::<_, String>(4),
                reserved1: 1,
                reserved2: 1,
                section: chara.section,
                class: chara.class,
                unk2: 0,
                symbol_chats: acc_info.symbol_chats.clone(),
                shortcuts: acc_info.shortcuts.clone(),
                autoreply: row.get::<_, String>(5),
                infoboard: row.get::<_, String>(6),
                unk3: vec![0; 0x001C],
                challenge_data: row.get::<_, Vec<u8>>(7),
                tech_menu: row.get::<_, Vec<u8>>(8),
                unk4: vec![0; 0x002C],
                quest_data2: row.get::<_, Vec<u8>>(9),
                key_config: key_config
            })
        })
        {
            Ok(v) => v,
            Err(e) => return Err(Error::BackendError(Some(Box::new(e))))
        };

        debug!("Character lookup results: {}", results.size_hint().0);

        match results.next() {
            Some(Ok(Ok(c))) => Ok(Some(c)),
            Some(Ok(Err(e))) => Err(Error::BackendError(Some(Box::new(e)))),
            Some(Err(e)) => Err(Error::BackendError(Some(Box::new(e)))),
            None => Ok(None)
        }
    }

    fn put_bb_character(&self, account_id: u32, slot: u8, chara: BbFullCharData, save_acct_data: bool) -> Result<()> {
        // First, we need the existing account information
        let mut acc_info = try_db!(self.fetch_bb_account_info(account_id)).unwrap();

        // Save the options, key, joy, shortcuts, symbols, to account data
        if save_acct_data {
            acc_info.key_config = chara.key_config.key_config.clone();
            acc_info.joy_config = chara.key_config.joy_config.clone();
            acc_info.symbol_chats = chara.symbol_chats.clone();
            acc_info.shortcuts = chara.shortcuts.clone();
            try!(self.put_bb_account_info(&acc_info));
        }

        // Check if a character exists
        let mut stmt = try_db!(self.conn.prepare("SELECT id FROM bb_character WHERE account_id=? AND slot=? LIMIT 1"));
        let exists;
        match stmt.query(&[&(account_id as i64), &(slot as i64)]) {
            Ok(mut rows) => {
                match rows.next() {
                    Some(Err(e)) => return Err(Error::BackendError(Some(Box::new(e)))),
                    Some(_) => {
                        info!("Character at {} exists for account {}; overwriting", slot, account_id);
                        exists = true;
                    },
                    None => {
                        exists = false;
                    }
                }
            },
            Err(e) => return Err(Error::BackendError(Some(Box::new(e))))
        }

        // Build the params array (bind the values to bind their lifetime for borrows)
        let account_id = account_id as i64;
        let slot = slot as i64;
        let inventory = serial_to_vec(&chara.inv);
        let char_data = serial_to_vec(&chara.chara);
        let bank = serial_to_vec(&chara.bank);

        let params: &[(&str, &ToSql)] = &[
            (":account_id", &account_id as &ToSql),
            (":slot", &slot),
            (":inventory", &inventory),
            (":char_data", &char_data),
            (":quest_data1", &chara.quest_data1),
            (":bank", &bank),
            (":guildcard_desc", &chara.guildcard_desc),
            (":autoreply", &chara.autoreply),
            (":infoboard", &chara.infoboard),
            (":challenge_data", &chara.challenge_data),
            (":tech_menu", &chara.tech_menu),
            (":quest_data2", &chara.quest_data2)
        ][..];

        // We will use update if a character exists in that slot already
        if exists {
            let mut stmt = try_db!(self.conn.prepare("UPDATE bb_character SET
                inventory = :inventory,
                char_data = :char_data,
                quest_data1 = :quest_data1,
                bank = :bank,
                guildcard_desc = :guildcard_desc,
                autoreply = :autoreply,
                infoboard = :infoboard,
                challenge_data = :challenge_data,
                tech_menu = :tech_menu,
                quest_data2 = :quest_data2
            WHERE account_id = :account_id AND slot = :slot"));
            try_db!(stmt.execute_named(params));
        } else {
            let mut stmt = try_db!(self.conn.prepare("INSERT INTO bb_character (
                account_id,
                slot,
                inventory,
                char_data,
                quest_data1,
                bank,
                guildcard_desc,
                autoreply,
                infoboard,
                challenge_data,
                tech_menu,
                quest_data2
            ) VALUES (
                :account_id,
                :slot,
                :inventory,
                :char_data,
                :quest_data1,
                :bank,
                :guildcard_desc,
                :autoreply,
                :infoboard,
                :challenge_data,
                :tech_menu,
                :quest_data2
            )"));
            try_db!(stmt.execute_named(params));
        }

        Ok(())
    }

    fn set_bb_login_flags(&self, account_id: u32, flags: u32) -> Result<()> {
        let mut stmt = try_db!(self.conn.prepare("INSERT OR UPDATE INTO bb_flags (account_id,login_flags) VALUES (?,?)"));
        let aid = account_id as i64;
        let f = flags as i64;
        try_db!(stmt.execute(&[&aid, &f]));
        Ok(())
    }

    fn get_bb_login_flags(&self, account_id: u32) -> Result<u32> {
        let mut stmt = try_db!(self.conn.prepare("SELECT login_flags FROM bb_flags WHERE account_id=?"));
        let aid = account_id as i64;
        let mut results = try_db!(stmt.query_map(&[&aid], |row| {
            row.get::<_, i64>(0)
        }));
        match results.next() {
            Some(Ok(f)) => Ok(f as u32),
            Some(Err(e)) => Err(Error::BackendError(Some(Box::new(e)))),
            None => Ok(0)
        }
    }
}

fn serial_to_vec<S: Serial>(i: &S) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    i.serialize(&mut cursor).unwrap();
    cursor.into_inner()
}

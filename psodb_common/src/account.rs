//! Structs related to accounts.

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use rand::random;

/// A struct representing a Blue Burst user's account.
pub struct Account {
    pub id: Option<u32>,
    pub username: String,
    pub password_hash: String,
    pub password_invalidated: bool,
    pub banned: bool
}

impl Account {
    pub fn new<U, P, S>(username: U, password: P, salt: S) -> Account
        where U: Into<String>, P: Into<String>, S: Into<String> {
        let un = username.into();
        let pw = password.into();
        let s = salt.into();
        Account {
            id: None,
            password_hash: hash_password(&un, &pw, &s),
            username: un,
            password_invalidated: false,
            banned: false
        }
    }

    /// Set the username for this account. This will invalidate the current password,
    /// because password hashes are salted by username and a salt.
    pub fn set_username<U: Into<String>>(&mut self, un: U) -> () {
        self.password_invalidated = true;
        self.username = un.into();
    }

    /// Set the password for this account.
    pub fn set_password<P: Into<String>, S: Into<String>>(&mut self, pw: P, salt: S) -> () {
        self.password_hash = hash_password(&self.username, &pw.into(), &salt.into())
    }

    /// Get the database ID of this account.
    pub fn id(&self) -> Option<u32> {
        self.id
    }

    pub fn cmp_password(&self, pw: &str, salt: &str) -> bool {
        let hashed = hash_password(&self.username, pw, salt);
        hashed == self.password_hash
    }
}

/// Extended account information for Blue Burst.
pub struct BbAccountInfo {
    pub account_id: u32,
    pub guildcard_num: u32,
    pub team_id: u32
}

impl BbAccountInfo {
    pub fn new() -> BbAccountInfo {
        let bbgc = (random::<u32>() % 100000000) + 400000000;
        BbAccountInfo {
            account_id: 0,
            guildcard_num: bbgc,
            team_id: 0
        }
    }
}

impl Default for BbAccountInfo {
    fn default() -> BbAccountInfo {
        BbAccountInfo::new()
    }
}

/// Generate a password hash string.
///
/// The hash algorithm is Sha256 over the string "un:pw:salt".
pub fn hash_password(un: &str, pw: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(un);
    hasher.input_str(":");
    hasher.input_str(pw);
    hasher.input_str(":");
    hasher.input_str(salt);
    hasher.result_str()
}

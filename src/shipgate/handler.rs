use std::sync::Arc;

use psodb_common::pool::Pool;
use psodb_common::account::Account;
use psodb_common::account::BbAccountInfo;

use ::shipgate::msg::*;
use super::ClientCtx;

/// Substructure built to handle requests without borrowing the full service.
pub struct MsgHandler<'a> {
    pool: Arc<Pool>,
    _client: &'a mut ClientCtx
}

impl<'a> MsgHandler<'a> {
    pub fn new(pool: Arc<Pool>, client: &mut ClientCtx) -> MsgHandler {
        MsgHandler {
            pool: pool,
            _client: client
        }
    }

    pub fn handle_login_challenge(&mut self, m: BbLoginChallenge) -> Message {
        let BbLoginChallenge { username, password } = m;

        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error getting pool connection: {:?}", e);
                return BbLoginChallengeAck { status: 1, account_id: 0 }.into() // unknown error occurred
            }
        };

        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbLoginChallengeAck { status: 1, account_id: 0 }.into()
            }
        };
        let account: Account = match handle.get_account_by_username(&username) {
            Ok(Some(a)) => a,
            Ok(None) => return BbLoginChallengeAck { status: 8, account_id: 0 }.into(), // no user exists
            Err(e) => {
                error!("Database error getting account: {:?}", e);
                return BbLoginChallengeAck { status: 1, account_id: 0 }.into() // unknown error occurred
            }
        };

        if (account.password_invalidated && !account.banned) || !account.cmp_password(&password, "") {
            return BbLoginChallengeAck { status: 2, account_id: 0 }.into()
        }

        if account.banned {
            info!("User {} is banned and attempted to log in.", username);
            return BbLoginChallengeAck { status: 6, account_id: 0 }.into()
        }

        BbLoginChallengeAck { status: 0, account_id: account.id().unwrap() }.into()
    }

    pub fn handle_get_bb_account_info(&mut self, m: BbGetAccountInfo) -> Message {
        let BbGetAccountInfo { account_id } = m;

        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetAccountInfoAck::default().into()
            }
        };

        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetAccountInfoAck::default().into()
            }
        };

        let info: BbAccountInfo = match handle.fetch_bb_account_info(account_id) {
            Ok(Some(a)) => a,
            Ok(None) => {
                error!("Account doesn't exist");
                return BbGetAccountInfoAck::default().into()
            },
            Err(e) => {
                error!("Database error getting Bb account info: {:?}", e);
                return BbGetAccountInfoAck::default().into()
            }
        };

        BbGetAccountInfoAck {
            status: 0,
            account_id: info.account_id,
            guildcard_num: info.guildcard_num,
            team_id: info.team_id,
            options: info.options,
            key_config: info.key_config.clone(),
            joy_config: info.joy_config.clone()
        }.into()
    }

    pub fn handle_bb_update_options(&mut self, m: BbUpdateOptions) {
        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let mut info: BbAccountInfo = match handle.fetch_bb_account_info(m.account_id) {
            Ok(Some(a)) => a,
            Ok(None) => {
                error!("Account doesn't exist; not updating account options");
                return
            },
            Err(e) => {
                error!("Database error getting Bb account info: {:?}", e);
                return
            }
        };

        info.options = m.options;
        match handle.put_bb_account_info(&info) {
            Ok(_) => (),
            Err(e) => {
                error!("Couldn't update account options: {:?}", e);
                return
            }
        }
    }

    pub fn handle_bb_update_keys(&mut self, m: BbUpdateKeys) {
        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let mut info: BbAccountInfo = match handle.fetch_bb_account_info(m.account_id) {
            Ok(Some(a)) => a,
            Ok(None) => {
                error!("Account doesn't exist; not updating account key config");
                return
            },
            Err(e) => {
                error!("Database error getting Bb account info: {:?}", e);
                return
            }
        };

        info.key_config = m.key_config;
        match handle.put_bb_account_info(&info) {
            Ok(_) => (),
            Err(e) => {
                error!("Couldn't update account options: {:?}", e);
                return
            }
        }
    }

    pub fn handle_bb_update_joy(&mut self, m: BbUpdateJoy) {
        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return
            }
        };
        let mut info: BbAccountInfo = match handle.fetch_bb_account_info(m.account_id) {
            Ok(Some(a)) => a,
            Ok(None) => {
                error!("Account doesn't exist; not updating account joystick config");
                return
            },
            Err(e) => {
                error!("Database error getting Bb account info: {:?}", e);
                return
            }
        };

        info.joy_config = m.joy_config;
        match handle.put_bb_account_info(&info) {
            Ok(_) => (),
            Err(e) => {
                error!("Couldn't update account options: {:?}", e);
                return
            }
        }
    }
}

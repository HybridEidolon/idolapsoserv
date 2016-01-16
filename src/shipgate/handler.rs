use std::sync::Arc;

use psodb_common::pool::Pool;
use psodb_common::account::Account;
use psodb_common::account::BbAccountInfo;
use psodata::chara::BbFullCharData;

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
            joy_config: info.joy_config.clone(),
            shortcuts: info.shortcuts.clone(),
            symbol_chats: info.symbol_chats.clone()
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

    pub fn handle_bb_get_character(&mut self, m: BbGetCharacter) -> Message {
        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetCharacterAck {
                    status: 1,
                    account_id: 0,
                    slot: 0,
                    full_char: None
                }.into()
            }
        };
        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetCharacterAck {
                    status: 2,
                    account_id: 0,
                    slot: 0,
                    full_char: None
                }.into()
            }
        };
        info!("Fetching character {} for account {} from database", m.slot, m.account_id);
        let chara: Option<BbFullCharData> = match handle.fetch_bb_character(m.account_id, m.slot) {
            Ok(a) => a,
            Err(e) => {
                error!("Database error getting character: {:?}", e);
                return BbGetCharacterAck {
                    status: 3,
                    account_id: 0,
                    slot: 0,
                    full_char: None
                }.into()
            }
        };

        BbGetCharacterAck {
            status: 0,
            account_id: m.account_id,
            slot: m.slot,
            full_char: chara
        }.into()
    }

    pub fn handle_bb_put_character(&mut self, m: BbPutCharacter) {
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
        let BbPutCharacter { account_id, slot, full_char, save_acct_data } = m;
        match handle.put_bb_character(account_id, slot, full_char, save_acct_data > 0) {
            Ok(_) => (),
            Err(e) => {
                error!("Database error putting character slot {} for account {}: {}", slot, account_id, e);
                return
            }
        }
    }

    pub fn handle_bb_set_login_flags(&mut self, m: BbSetLoginFlags) {
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
        match handle.set_bb_login_flags(m.account_id, m.flags) {
            Ok(_) => (),
            Err(e) => {
                error!("Database error setting login flags: {:?}", e);
                return
            }
        }
    }

    pub fn handle_bb_get_login_flags(&mut self, m: BbGetLoginFlags) -> Message {
        let a = match self.pool.get_connection() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetLoginFlagsAck {
                    status: 1,
                    account_id: 0,
                    flags: 0
                }.into()
            }
        };
        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetLoginFlagsAck {
                    status: 2,
                    account_id: 0,
                    flags: 0
                }.into()
            }
        };
        match handle.get_bb_login_flags(m.account_id) {
            Ok(flags) => BbGetLoginFlagsAck {
                status: 0,
                account_id: m.account_id,
                flags: flags
            }.into(),
            Err(e) => {
                error!("Database error getting login flags: {:?}", e);
                BbGetLoginFlagsAck {
                    status: 3,
                    account_id: 0,
                    flags: 0
                }.into()
            }
        }
    }
}

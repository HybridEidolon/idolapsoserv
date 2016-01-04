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
                return BbGetAccountInfoAck { status: 1, account_id: account_id, guildcard_num: 0, team_id: 0 }.into()
            }
        };

        let handle = match a.lock() {
            Ok(h) => h,
            Err(e) => {
                error!("Database error locking connection handle: {:?}", e);
                return BbGetAccountInfoAck { status: 2, account_id: account_id, guildcard_num: 0, team_id: 0 }.into()
            }
        };

        let info: BbAccountInfo = match handle.fetch_bb_account_info(account_id) {
            Ok(Some(a)) => a,
            Ok(None) => unreachable!(),
            Err(e) => {
                error!("Database error getting Bb account info: {:?}", e);
                return BbGetAccountInfoAck { status: 3, account_id: account_id, guildcard_num: 0, team_id: 0 }.into()
            }
        };

        BbGetAccountInfoAck {
            status: 0,
            account_id: info.account_id,
            guildcard_num: info.guildcard_num,
            team_id: info.team_id
        }.into()
    }
}

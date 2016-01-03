use std::sync::Arc;

use psodb_common::pool::Pool;
use psodb_common::account::Account;

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
}

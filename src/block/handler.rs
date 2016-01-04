use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use mio::Sender;

use psomsg::bb::*;

use ::shipgate::client::callbacks::SgCbMgr;
use ::loop_handler::LoopMsg;
use ::shipgate::msg::Message as Sgm;
use ::shipgate::msg::BbLoginChallenge;
use ::shipgate::msg::BbGetAccountInfo;

use super::client::ClientState;

pub struct BlockHandler {
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BlockHandler>,
    client_id: usize,
    clients: Rc<RefCell<HashMap<usize, ClientState>>>
}

impl BlockHandler {
    pub fn new(sender: Sender<LoopMsg>,
               sg_sender: SgCbMgr<BlockHandler>,
               client_id: usize,
               clients: Rc<RefCell<HashMap<usize, ClientState>>>) -> BlockHandler {
        BlockHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id,
            clients: clients
        }
    }

    pub fn bb_login(&mut self, m: BbLogin) {
        let sec_data = m.security_data.clone();
        // Security data should be set when connecting to the Ship (sent by Login)
        // Drop if it's invalid.
        if sec_data.magic != 0xCAFEB00B {
            let m = Message::LargeMsg(0, LargeMsg("Invalid security data".to_string()));
            self.sender.send((self.client_id, m).into()).unwrap();
            self.sender.send(LoopMsg::DropClient(self.client_id)).unwrap();
            return
        }

        let sgm = BbLoginChallenge { username: m.username.clone(), password: m.password.clone() };
        self.sg_sender.request(self.client_id, sgm, move|mut h, m| {
            // We need the extended BB account data.
            if let Sgm::BbLoginChallengeAck(_, a) = m {
                if a.status != 0 {
                    // The shipgate says this account isn't usable for whatever reason. Drop.
                    let r = Message::BbSecurity(0, BbSecurity {
                        err_code: a.status,
                        tag: 0,
                        guildcard: 0,
                        team_id: 0,
                        security_data: sec_data.clone(),
                        caps: 0
                    });
                    h.sender.send((h.client_id, r).into()).unwrap();
                    h.sender.send(LoopMsg::DropClient(h.client_id)).unwrap();
                    return
                }

                let sec_data = sec_data.clone();

                let sgm: Sgm = BbGetAccountInfo { account_id: a.account_id }.into();
                h.sg_sender.request(h.client_id, sgm, move|h, m| {
                    if let Sgm::BbGetAccountInfoAck(_, a) = m {
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 0,
                            tag: 0x00010000,
                            guildcard: a.guildcard_num,
                            team_id: a.team_id,
                            security_data: sec_data,
                            caps: 0x00000102
                        });
                        h.sender.send((h.client_id, r).into()).unwrap();

                        {
                            let mut b = h.clients.borrow_mut();
                            let ref mut c = b.get_mut(&h.client_id).unwrap();
                            c.sec_data = sec_data.clone();
                            c.team_id = a.team_id;
                            c.bb_guildcard = a.guildcard_num;
                        }

                        // TODO move into a lobby
                        return
                    }
                }).unwrap();
            } else {
                warn!("Unexpected response from shipgate: {:?}", m);
                h.sender.send(LoopMsg::DropClient(h.client_id)).unwrap();
                return
            }
        }).unwrap();
    }
}

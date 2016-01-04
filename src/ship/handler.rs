use ::loop_handler::LoopMsg;

use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::net::SocketAddrV4;

use mio::Sender;

use psomsg::bb::*;

use ::shipgate::client::callbacks::SgCbMgr;
use ::shipgate::msg::{BbLoginChallenge,
    //BbLoginChallengeAck,
    BbGetAccountInfo,
    ShipList as SgShipList,
    ShipListAck,
    Message as Sgm};

use super::client::ClientState;

pub struct ShipHandler {
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<ShipHandler>,
    client_id: usize,
    clients: Rc<RefCell<HashMap<usize, ClientState>>>,
    param_data: Rc<(Message, Vec<Message>)>
}

impl ShipHandler {
    pub fn new(sender: Sender<LoopMsg>, sg_sender: SgCbMgr<ShipHandler>, client_id: usize, clients: Rc<RefCell<HashMap<usize, ClientState>>>, param_data: Rc<(Message, Vec<Message>)>) -> ShipHandler {
        ShipHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id,
            clients: clients,
            param_data: param_data
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
                h.sg_sender.request(h.client_id, sgm, move|mut h, m| {
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

                        // If they have selected their character, we are at ship select.
                        if sec_data.sel_char {
                            info!("Sending timestamp to {}", h.client_id);
                            let r = Message::Timestamp(0, Timestamp {
                                year: 2016,
                                month: 1,
                                day: 1,
                                hour: 0,
                                minute: 30,
                                second: 30,
                                msec: 0
                            });
                            h.sender.send((h.client_id, r).into()).unwrap();
                            let sgm: Sgm = SgShipList.into();
                            h.sg_sender.request(h.client_id, sgm, move|mut h, m| h.sg_shiplist(m)).unwrap();
                        }
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

    pub fn sg_shiplist(&mut self, m: Sgm) {
        info!("Sending ship list to {}", self.client_id);
        if let Sgm::ShipListAck(_, ShipListAck(ships)) = m {
            let ships: Vec<(SocketAddrV4, String)> = ships;
            let mut shiplist: Vec<ShipListItem> = Vec::new();
            shiplist.push(ShipListItem {
                menu_id: 0,
                item_id: 0,
                flags: 0x0000,
                name: "".to_string()
            });
            let mut i = 1;
            for (_, name) in ships.into_iter() {
                shiplist.push(ShipListItem {
                    menu_id: 0,
                    item_id: i,
                    flags: 0x0F04,
                    name: name.clone()
                });
                i += 1;
            }
            let r = Message::ShipList((shiplist.len() - 1) as u32, ShipList(shiplist));
            self.sender.send((self.client_id, r).into()).unwrap();
        }
    }

    pub fn bb_option_request(&mut self) {
        // Send the key bindings for this account.
        // TODO save and restore from shipgate
        let r = Message::BbOptionConfig(0, BbOptionConfig(BbTeamAndKeyData::default()));
        self.sender.send((self.client_id, r).into()).unwrap();
    }

    pub fn bb_checksum(&mut self, m: BbChecksum) {
        info!("Client {}'s checksum is {:x}", self.client_id, m.0);
        let r = Message::BbChecksumAck(0, BbChecksumAck(true));
        self.sender.send((self.client_id, r).into()).unwrap();
    }

    pub fn bb_guildcard_req(&mut self) {
        use crc::crc32::checksum_ieee as checksum;

        // TODO actual guildcard data
        let checksum = checksum(&vec![0u8; 54672]);
        let r = Message::BbGuildCardHdr(0, BbGuildCardHdr {
            one: 1,
            len: 54672,
            checksum: checksum
        });
        self.sender.send((self.client_id, r).into()).unwrap();
    }

    pub fn bb_guildcard_chunk_req(&mut self, m: BbGuildCardChunkReq) {
        let BbGuildCardChunkReq(_, chunk, cont) = m;
        if cont {
            let size_remaining: usize = 54672 - (chunk as usize * 0x6800);
            let size: usize = if size_remaining < 0x6800 { size_remaining } else { 0x6800 };
            info!("Sending guild card chunk {} of size {}", chunk, size);
            let r = Message::BbGuildCardChunk(0, BbGuildCardChunk {
                unk: 0,
                chunk: chunk,
                data: vec![0u8; size]
            });
            self.sender.send((self.client_id, r).into()).unwrap();
        }
    }

    pub fn bb_char_select(&mut self, m: BbCharSelect) {
        let BbCharSelect { slot, selecting } = m;
        if selecting {
            // They are selecting an existing character slot.
            let r = Message::LargeMsg(0, LargeMsg("how'd you do that. you can't even make characters yet. get out.".to_string()));
            self.sender.send((self.client_id, r).into()).unwrap();
        } else {
            // They want information about a character slot.
            let r = Message::BbCharAck(0, BbCharAck {
                slot: slot,
                code: 2 //nonexistant
            });
            self.sender.send((self.client_id, r).into()).unwrap();
        }
    }

    pub fn bb_param_hdr_req(&mut self) {
        self.sender.send((self.client_id, self.param_data.0.clone()).into()).unwrap();
    }

    pub fn bb_param_chunk_req(&mut self, chunk: u32) {
        let chunks = self.param_data.1.len();
        if let Some(ref a) = self.param_data.1.get(chunk as usize) {
            info!("Sending param chunk {} of {}", chunk, chunks);
            self.sender.send((self.client_id, (*a).clone()).into()).unwrap();
        } else {
            info!("Client requested invalid param chunk.");
            let m = Message::LargeMsg(0, LargeMsg("Whoops, you requested a chunk in the param table that doesn't exist.".to_string()));
            self.sender.send((self.client_id, m).into()).unwrap();
        }
    }

    pub fn bb_char_info(&mut self, m: BbCharInfo) {
        let BbCharInfo(slot, chardata) = m;

        info!("Character created: {}", chardata.name);
        // TODO persist this character to shipgate
        let sec_data;
        let bb_guildcard;
        let team_id;
        {
            let mut b = self.clients.borrow_mut();
            let c = b.get_mut(&self.client_id).unwrap();
            c.sec_data.slot = slot as u8;
            c.sec_data.sel_char = true;
            sec_data = c.sec_data.clone();
            bb_guildcard = c.bb_guildcard;
            team_id = c.team_id;
        }

        let r = Message::BbSecurity(0, BbSecurity {
            err_code: 0,
            tag: 0x00010000,
            guildcard: bb_guildcard,
            team_id: team_id,
            security_data: sec_data.clone(),
            caps: 0x00000102
        });
        self.sender.send((self.client_id, r).into()).unwrap();

        let r = Message::BbCharAck(0, BbCharAck {slot: slot, code: 0});
        self.sender.send((self.client_id, r).into()).unwrap();
    }
}

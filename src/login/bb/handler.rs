use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddrV4;

use mio::Sender;

use psomsg::bb::*;

use ::shipgate::client::callbacks::SgCbMgr;
use ::shipgate::msg::Message as Sgm;
use ::shipgate::msg::{
    BbLoginChallenge,
    BbGetAccountInfo,
    ShipList as SgShipList,
    ShipListAck
};
use ::loop_handler::LoopMsg;

use super::client::ClientState;

pub struct BbLoginHandler {
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BbLoginHandler>,
    client_id: usize,
    clients: Rc<RefCell<HashMap<usize, ClientState>>>,
    param_files: Arc<(Message, Vec<Message>)>,
    redir_addr: SocketAddrV4
}

impl BbLoginHandler {
    pub fn new(sender: Sender<LoopMsg>, redir_addr: SocketAddrV4, sg_sender: SgCbMgr<BbLoginHandler>, client_id: usize, clients: Rc<RefCell<HashMap<usize, ClientState>>>, param_files: Arc<(Message, Vec<Message>)>) -> BbLoginHandler {
        BbLoginHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id,
            clients: clients,
            param_files: param_files,
            redir_addr: redir_addr
        }
    }

    pub fn bb_login(&mut self, m: BbLogin) {
        // on this server, we need to contact the shipgate
        // and verify credentials, then forward to any of
        // the ships for the character step.
        let sec_data = m.security_data.clone();
        info!("Blue Burst Login challenge: {:?}", m);
        let sm = BbLoginChallenge { username: m.username.clone(), password: m.password.clone() };
        self.sg_sender.request(self.client_id, sm, move|mut h, sm| {
            if let Sgm::BbLoginChallengeAck(_, sm) = sm {
                if sm.status != 0 {
                    let r = Message::BbSecurity(0, BbSecurity {
                        err_code: sm.status,
                        tag: 0x00010000,
                        guildcard: 0,
                        team_id: 0,
                        security_data: Default::default(),
                        caps: 0x00000101
                    });
                    h.sender.send((h.client_id, r).into()).unwrap();
                    return
                }

                // get BB extended account data
                let sec_data = sec_data.clone();
                h.sg_sender.request(h.client_id, BbGetAccountInfo { account_id: sm.account_id }, move|mut h, sm| {
                    if let Sgm::BbGetAccountInfoAck(_, sm) = sm {
                        let sec_data = sec_data.clone();
                        // First of all, if account acquisition failed, we should disconnect immediately.
                        if sm.status != 0 {
                            let r = Message::LargeMsg(0, LargeMsg("Internal DB error.".to_string()));
                            h.sender.send((h.client_id, r).into()).unwrap();
                            h.sender.send(LoopMsg::DropClient(h.client_id)).unwrap();
                            return
                        }

                        // If the magic code in the client's security data is 0, we need to redirect to self
                        if sec_data.magic != 0xCAFEB00B {
                            info!("Client {} is fresh {:?}", h.client_id, sec_data);
                            let mut sec_data: BbSecurityData = Default::default();
                            sec_data.magic = 0xCAFEB00B;

                            // TODO find out if the account is logged in and disconnect if it is.

                            {
                                let mut b = h.clients.borrow_mut();
                                let mut c = b.get_mut(&h.client_id).unwrap();
                                c.sec_data = sec_data.clone();
                                c.team_id = sm.team_id;
                                c.bb_guildcard = sm.guildcard_num;
                            }

                            let r = Message::BbSecurity(0, BbSecurity {
                                err_code: 0,
                                tag: 0x00010000,
                                guildcard: sm.guildcard_num,
                                team_id: 0xFFFFFFFF,
                                security_data: sec_data,
                                caps: 0x00000101
                            });
                            h.sender.send((h.client_id, r).into()).unwrap();

                            // TODO redirect to actual self IP (need external facing IP)
                            let r = Message::Redirect(0, Redirect {
                                ip: *h.redir_addr.ip(),
                                port: h.redir_addr.port()
                            });
                            h.sender.send((h.client_id, r).into()).unwrap();
                        } else {
                            // Client already has a session; we'll capture their security data.
                            info!("Existing session on client with sec data: {:?}", sec_data);
                            {
                                let mut b = h.clients.borrow_mut();
                                let mut c = b.get_mut(&h.client_id).unwrap();
                                c.sec_data = sec_data.clone();
                                c.bb_guildcard = sm.guildcard_num;
                                c.team_id = sm.team_id;
                            }

                            let r = Message::BbSecurity(0, BbSecurity {
                                err_code: 0,
                                tag: 0x00010000,
                                guildcard: sm.guildcard_num,
                                team_id: 0xFFFFFFFF,
                                security_data: sec_data.clone(),
                                caps: 0x00000101
                            });
                            h.sender.send((h.client_id, r).into()).unwrap();

                            // If they've selected a character, they want the ship list now.
                            if sec_data.sel_char != 0 {
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
                                h.sg_sender.request(h.client_id, SgShipList, move|mut h, m| h.sg_shiplist_ack(m)).unwrap();
                            }
                        }
                    }
                }).unwrap();
            }
        }).unwrap();
    }

    pub fn sg_shiplist_ack(&mut self, m: Sgm) {
        if let Sgm::ShipListAck(_, ShipListAck(ships)) = m {
            let ships: Vec<(SocketAddrV4, String)> = ships;
            {
                let mut b = self.clients.borrow_mut();
                let mut c = b.get_mut(&self.client_id).unwrap();
                c.ships = Some(ships.clone());
            }
            let mut shiplist: Vec<ShipListItem> = Vec::new();
            shiplist.push(ShipListItem {
                menu_id: 0,
                item_id: 0,
                flags: 0x0004,
                name: "SHIP/US".to_string()
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
            debug!("Sending guild card chunk {} of size {}", chunk, size);
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
            // TODO
            let mut b = self.clients.borrow_mut();
            let mut c = b.get_mut(&self.client_id).unwrap();
            c.sec_data.sel_char = 1;
            let r = Message::BbSecurity(0, BbSecurity {
                err_code: 0,
                tag: 0x00010000,
                guildcard: c.bb_guildcard,
                team_id: 0xFFFFFFFF,
                security_data: c.sec_data.clone(),
                caps: 0x00000101
            });
            self.sender.send((self.client_id, r).into()).unwrap();
            let r = Message::BbCharAck(0, BbCharAck {
                slot: slot,
                code: 0
            });
            //let r = Message::LargeMsg(0, LargeMsg("how'd you do that. you can't even make characters yet. get out.".to_string()));
            self.sender.send((self.client_id, r).into()).unwrap();
        } else {
            // They want information about a character slot.
            let r = Message::BbCharInfo(0, BbCharInfo(slot, BbMiniCharData::default()));
            // let r = Message::BbCharAck(0, BbCharAck {
            //     slot: slot,
            //     code: 2 //exists?
            // });
            self.sender.send((self.client_id, r).into()).unwrap();
        }
    }

    pub fn bb_param_hdr_req(&mut self) {
        self.sender.send((self.client_id, self.param_files.0.clone()).into()).unwrap();
    }

    pub fn bb_param_chunk_req(&mut self, chunk: u32) {
        let chunks = self.param_files.1.len();
        if let Some(ref a) = self.param_files.1.get(chunk as usize) {
            debug!("Sending param chunk {} of {}", chunk, chunks);
            self.sender.send((self.client_id, (*a).clone()).into()).unwrap();
        } else {
            warn!("Client requested invalid param chunk.");
            let m = Message::LargeMsg(0, LargeMsg("Whoops, you requested a chunk in the param table that doesn't exist.".to_string()));
            self.sender.send((self.client_id, m).into()).unwrap();
        }
    }

    pub fn bb_char_info(&mut self, m: BbCharInfo) {
        let BbCharInfo(slot, chardata) = m;

        info!("Character created: {:?}", chardata);

        // TODO persist this character to shipgate
        let sec_data;
        let bb_guildcard;
        //let team_id;
        {
            let mut b = self.clients.borrow_mut();
            let c = b.get_mut(&self.client_id).unwrap();
            c.sec_data.slot = slot as u8;
            c.sec_data.sel_char = 1;
            c.sec_data.magic = 0xCAFEB00B;
            sec_data = c.sec_data.clone();
            bb_guildcard = c.bb_guildcard;
            //team_id = c.team_id;
        }

        let r = Message::BbSecurity(0, BbSecurity {
            err_code: 0,
            tag: 0x00010000,
            guildcard: bb_guildcard,
            team_id: 0xFFFFFFFF,
            security_data: sec_data.clone(),
            caps: 0x00000101
        });
        self.sender.send((self.client_id, r).into()).unwrap();

        let r = Message::BbCharAck(0, BbCharAck {slot: slot, code: 0});
        self.sender.send((self.client_id, r).into()).unwrap();
    }

    pub fn menu_select(&mut self, m: MenuSelect) {
        let MenuSelect(menu, item) = m;

        match menu {
            0 => {
                info!("Client selected a ship; redirect");
                let ships;
                {
                    let mut b = self.clients.borrow_mut();
                    let c = b.get_mut(&self.client_id).unwrap();
                    ships = c.ships.clone();
                }

                // Which ship did they select? item - 1 = idx
                if let Some(shiplist) = ships {
                    let ship = shiplist.get(item as usize - 1);
                    if let Some(ship) = ship {
                        let r = Message::Redirect(0, Redirect {
                            ip: ship.0.ip().clone(),
                            port: ship.0.port()
                        });
                        self.sender.send((self.client_id, r).into()).unwrap();
                    } else {
                        // invalid menu item
                        self.sender.send(LoopMsg::DropClient(self.client_id)).unwrap();
                    }
                }
            },
            _ => {
                let r = Message::LargeMsg(0, LargeMsg("Invalid menu".to_string()));
                self.sender.send((self.client_id, r).into()).unwrap();
                self.sender.send(LoopMsg::DropClient(self.client_id)).unwrap();
            }
        }
    }
}

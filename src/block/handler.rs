use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
//use std::fs::File;

use mio::Sender;

use psomsg::bb::*;

//use ::game::CharClass;
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
                            team_id: 0xFFFFFFFF,
                            security_data: sec_data.clone(),
                            caps: 0x00000101
                        });
                        h.sender.send((h.client_id, r).into()).unwrap();

                        let mut b = h.clients.borrow_mut();
                        let ref mut c = b.get_mut(&h.client_id).unwrap();
                        c.sec_data = sec_data.clone();
                        c.team_id = a.team_id;
                        c.bb_guildcard = a.guildcard_num;

                        let mut fc: BbFullCharData;
                        {
                            {
                                let mut ll: Vec<(u32, u32)> = Vec::new();
                                ll.push((60, 1));
                                ll.push((60, 2));
                                ll.push((60, 3));
                                ll.push((60, 4));
                                ll.push((60, 5));
                                ll.push((60, 6));
                                ll.push((60, 7));
                                ll.push((60, 8));
                                ll.push((60, 9));
                                ll.push((60, 10));
                                ll.push((60, 11));
                                ll.push((60, 12));
                                ll.push((60, 13));
                                ll.push((60, 14));
                                ll.push((60, 15));
                                ll.push((0, 0));
                                let r = Message::LobbyList(15, LobbyList { items: ll });
                                h.sender.send((h.client_id, r).into()).unwrap();
                            }
                            // fc = ::util::nsc::read_nsc(&mut File::open("data/default/default_0.nsc").unwrap(), CharClass::HUmar).unwrap();
                            fc = BbFullCharData::default();
                            // {
                            //     use std::fs::File;
                            //     use psoserial::Serial;
                            //     let mut file = File::open("character.bin").unwrap();
                            //     if let Message::BbFullChar(_, BbFullChar(cha)) = Serial::deserialize(&mut file).unwrap() {
                            //         fc = cha;
                            //     } else {
                            //         info!("had to default");
                            //         fc = Default::default();
                            //     }
                            //     info!("{:?}", fc);
                            // }
                            // monomates in their inventory
                            // fc.inv.item_count = 255;
                            // fc.inv.hp_mats = 255;
                            // fc.inv.tp_mats = 255;
                            // fc.inv.lang = 255;
                            // for item in fc.inv.items.iter_mut() {
                            //     item.exists = 0xFF00;
                            //     item.data.item_id = 0xFFFFFFFF;
                            // }
                            // for item in fc.bank.items.iter_mut() {
                            //     item.data.item_id = 0xFFFFFFFF;
                            // }
                            // fc.inv.hp_mats = 255;
                            // fc.inv.tp_mats = 127;
                            // fc.inv.items[0].exists = 0x01;
                            // fc.inv.items[0].flags = 0;
                            // fc.inv.items[0].data.data[1] = 0x01;
                            // fc.inv.items[0].data.item_id = 0x00010000;
                            //
                            // fc.inv.items[1].exists = 0x01;
                            // fc.inv.items[1].flags = 0;
                            // fc.inv.items[1].data.data[1] = 0x01;
                            // fc.inv.items[1].data.item_id = 0x00010001;

                            fc.name = "Rico".to_string();
                            fc.chara.name = "Rico".to_string();
                            fc.guildcard = a.guildcard_num;
                            fc.chara.guildcard = format!("  {}", a.guildcard_num);
                            // fc.team_name = "\tEFlowen".to_string();
                            //fc.key_config = Default::default();
                            fc.key_config.team_id = 0;
                            // fc.key_config.team_name = fc.team_name.clone();
                            // fc.key_config.guildcard = a.guildcard_num;
                            fc.key_config.team_rewards = 0xFFFFFFFF;
                            fc.chara.level = 199;
                            fc.chara.stats.hp = 100;
                            fc.section = 3;
                            fc.class = 1;
                            fc.chara.section = 3;
                            fc.chara.class = 1;
                            fc.chara.model = 1;
                            fc.chara.model_flag = 8;
                            fc.chara.costume = 1;
                            fc.chara.skin = 1;
                            fc.chara.head = 1;
                            fc.chara.hair = 1;
                            fc.chara.hair_r = 0xFF;
                            fc.chara.hair_g = 0xFF;
                            fc.chara.hair_b = 0xFF;
                            fc.chara.prop_x = 0.3;
                            fc.chara.prop_y = 0.3;
                            fc.chara.play_time = 0xFFFFFFFF;
                            fc.autoreply = "".to_string();
                            fc.infoboard = "".to_string();


                            let r = Message::BbFullChar(0, BbFullChar(fc.clone()));
                            h.sender.send((h.client_id, r).into()).unwrap();
                            c.full_char = Some(fc);
                            let r = Message::CharDataRequest(0, CharDataRequest);
                            h.sender.send((h.client_id, r).into()).unwrap();
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

    pub fn bb_char_dat(&mut self, _m: BbCharDat) {
        info!("Client {} is joining lobby 1 <TODO>", self.client_id);

        let mut b = self.clients.borrow_mut();
        let c = b.get_mut(&self.client_id).unwrap();
        let fc;
        if let Some(ref fce) = c.full_char {
            fc = fce.clone();
        } else {
            return
        }

        let mut l = LobbyJoin::default();
        l.client_id = 0;
        l.leader_id = 1;
        l.one = 1;
        l.lobby_num = 0;
        l.block_num = 1;
        l.event = 12;
        let mut lm = LobbyMember::default();
        lm.hdr.guildcard = c.bb_guildcard;
        lm.hdr.tag = 0x00010000;
        lm.hdr.client_id = 0;
        lm.hdr.name = fc.name.clone();
        lm.data = fc.chara.clone();
        lm.inventory = fc.inv.clone();
        for item in lm.inventory.items.iter_mut() {
            item.data.item_id |= 0x10000000;
        }
        l.members.push(lm);

        //Message::LobbyArrowList(0, LobbyArrowList(Vec::new())).serialize(&mut w).unwrap();

        let r = Message::LobbyJoin(1, l);
        self.sender.send((self.client_id, r).into()).unwrap();

        // send this player's quest data, don't know why
        let r = Message::BbSubCmd60(0, BbSubCmd60::QuestData1 {
            client_id: 0,
            unused: 0,
            data: QuestData1(fc.quest_data1.clone())
        });
        self.sender.send((self.client_id, r).into()).unwrap();
    }

    pub fn bb_chat(&mut self, m: BbChat) {
        // TODO propogate
        let BbChat(gc, text) = m;
        info!("{}: {}", gc, text);
        let mut b = self.clients.borrow_mut();
        let c = b.get_mut(&self.client_id).unwrap();
        let message: Message = BbChat(c.bb_guildcard, text).into();
        self.sender.send((self.client_id, message).into()).unwrap();
    }

    pub fn bb_create_game(&mut self, m: BbCreateGame) {
        info!("{} creating game {}", self.client_id, m.name);
        let mut b = self.clients.borrow_mut();
        let c = b.get_mut(&self.client_id).unwrap();

        let mut l = BbGameJoin::default();
        l.client_id = 0;
        l.leader_id = 0;
        l.one = 1;
        l.one2 = 1;
        l.difficulty = m.difficulty;
        l.episode = m.episode;
        let mut ph = PlayerHdr::default();
        ph.tag = 0x00010000;
        ph.guildcard = c.bb_guildcard;
        ph.client_id = 0;
        ph.name = c.full_char.as_ref().unwrap().name.clone();
        l.players.push(ph);
        let r: Message = Message::BbGameJoin(0, l);
        self.sender.send((self.client_id, r).into()).unwrap();
    }
}

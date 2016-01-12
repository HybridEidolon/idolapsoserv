use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
//use std::fs::File;

use mio::Sender;

use psomsg::bb::*;

use psodata::battleparam::BattleParamTables;

//use ::game::CharClass;
use ::shipgate::client::callbacks::SgCbMgr;
use ::loop_handler::LoopMsg;
use ::shipgate::msg::Message as Sgm;
use ::shipgate::msg::BbLoginChallenge;
use ::shipgate::msg::BbGetAccountInfo;
use ::maps::Areas;

use super::client::ClientState;
use super::lobbyhandler::Lobby;
use super::partyhandler::Party;

const MENU_GAME_LIST: u32 = 0x00080000;

pub struct BlockHandler {
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BlockHandler>,
    pub client_id: usize,
    clients: Rc<RefCell<HashMap<usize, Rc<RefCell<ClientState>>>>>,
    lobbies: Rc<RefCell<Vec<Lobby>>>,
    parties: Rc<RefCell<Vec<Party>>>,
    pub battle_params: Arc<BattleParamTables>,
    online_maps: Arc<Areas>
}

impl BlockHandler {
    pub fn new(sender: Sender<LoopMsg>,
               sg_sender: SgCbMgr<BlockHandler>,
               client_id: usize,
               clients: Rc<RefCell<HashMap<usize, Rc<RefCell<ClientState>>>>>,
               lobbies: Rc<RefCell<Vec<Lobby>>>,
               parties: Rc<RefCell<Vec<Party>>>,
               battle_params: Arc<BattleParamTables>,
               online_maps: Arc<Areas>) -> BlockHandler {
        BlockHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id,
            clients: clients,
            lobbies: lobbies,
            parties: parties,
            battle_params: battle_params,
            online_maps: online_maps
        }
    }

    /// Get the client state for the given client ID. Be smart with RefCell borrows,
    /// the program will panic if you fail the borrow invariant checks (basically,
    /// don't borrow mutably unless you have to).
    pub fn get_client_state(&self, client: usize) -> Option<Rc<RefCell<ClientState>>> {
        self.clients.borrow().get(&client).map(|v| v.clone())
    }

    /// Send a message to a client.
    pub fn send_to_client(&self, client: usize, message: Message) {
        // no support for versions other than BB yet...
        self.sender.send((client, message.clone()).into()).unwrap();
    }

    /// Send a message and disconnect the client.
    pub fn send_fatal_error(&self, client: usize, msg: &str) {
        let m = Message::LargeMsg(0, LargeMsg(msg.to_string()));
        self.send_to_client(client, m);
        self.sender.send(LoopMsg::DropClient(client)).unwrap();
    }

    /// Send a non-fatal Msg1 to the client.
    pub fn send_error(&self, client: usize, msg: &str) {
        let m = Message::BbMsg1(0, BbMsg1(msg.to_string()));
        self.send_to_client(client, m);
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

                        let cr = h.get_client_state(h.client_id).unwrap();
                        let ref mut c = cr.borrow_mut();
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
                            fc.inv.item_count = 2;
                            fc.inv.hp_mats = 255;
                            fc.inv.tp_mats = 255;
                            // fc.inv.lang = 255;
                            for item in fc.inv.items.iter_mut() {
                                item.exists = 0xFF00;
                                item.data.item_id = 0xFFFFFFFF;
                            }
                            for item in fc.bank.items.iter_mut() {
                                item.data.item_id = 0xFFFFFFFF;
                            }
                            // fc.inv.hp_mats = 255;
                            // fc.inv.tp_mats = 127;
                            fc.inv.items[0].exists = 0x01;
                            fc.inv.items[0].flags = 0x8;
                            fc.inv.items[0].data.data[1] = 0x01;
                            fc.inv.items[0].data.item_id = 0x00010000;

                            fc.inv.items[1].exists = 0x01;
                            fc.inv.items[1].flags = 0;
                            fc.inv.items[1].data.data[1] = 0x01;
                            fc.inv.items[1].data.item_id = 0x00010001;

                            fc.name = "Rico".to_string();
                            fc.chara.name = "Rico".to_string();
                            fc.guildcard = a.guildcard_num;
                            fc.chara.guildcard = format!("  {}", a.guildcard_num);
                            // fc.team_name = "\tEFlowen".to_string();
                            //fc.key_config = Default::default();
                            fc.key_config.team_id = 0;
                            // fc.key_config.team_name = fc.team_name.clone();
                            // fc.key_config.guildcard = a.guildcard_num;
                            //fc.key_config.team_rewards = 0xFFFFFFFF;
                            // fc.chara.level = 199;
                            fc.chara.stats.hp = 3000;
                            fc.chara.stats.atp = 3000;
                            fc.chara.stats.dfp = 3000;
                            fc.chara.stats.evp = 3000;
                            fc.chara.stats.ata = 3000;
                            fc.chara.stats.mst = 3000;
                            fc.chara.stats.lck = 3000;
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
        // They are joining a lobby now. Find an empty lobby.
        let lr = self.lobbies.clone();
        let ref mut lobbies = lr.borrow_mut();

        for l in lobbies.iter_mut() {
            if !l.is_full() {
                let cid = self.client_id;
                l.add_player(self, cid).unwrap();
                return
            }
        }

        info!("Unable to add client {} to a lobby because they're all full.", self.client_id);
        self.send_fatal_error(self.client_id, "All lobbies on this block are full. Please try again.");
    }

    pub fn bb_chat(&mut self, mut m: BbChat) {
        let gc_num;
        let player_name;

        {
            let cr = self.get_client_state(self.client_id).unwrap();
            let ref c = cr.borrow();
            gc_num = c.bb_guildcard;
            player_name = c.full_char.as_ref().unwrap().chara.name.clone();
        }
        // First, we'll check if they're in a lobby.
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                if l.has_player(self.client_id) {
                    info!("<{}:{}> {}: {}", l.block_num(), l.lobby_num() + 1, player_name, m.1);
                    m.0 = gc_num;
                    l.bb_broadcast(self, None, m.into()).unwrap();
                    return
                }
            }
        }
    }

    pub fn bb_create_game(&mut self, m: BbCreateGame) {
        info!("Client {} is creating party {}", self.client_id, &m.name[2..]);

        let psr = self.parties.clone();
        let mut parties = psr.borrow_mut();
        // check if a party with that name already exists
        for p in parties.iter_mut() {
            if p.name == m.name {
                self.send_error(self.client_id, "\tEA party with that\nname already exists.");
                return
            }
        }

        // validate based on server support
        info!("Party is for episode {}", m.episode);
        if m.challenge != 0 && m.episode == 3 {
            self.send_error(self.client_id, "\tEChallenge mode is not supported\non Episode 4.\nOnly Episode 1 and 2 have\nChallenge mode.");
            return
        }
        if m.battle != 0 {
            self.send_error(self.client_id, "\tEBattle mode is not\nsupported yet. Sorry!");
            return
        }
        if m.challenge != 0 {
            self.send_error(self.client_id, "\tEChallenge mode is not\nsupported yet. Sorry!");
            return
        }
        if m.single_player != 0 {
            self.send_error(self.client_id, "\tEOffline mode maps are\nnot loaded yet. WIP");
            return
        }

        // we first want to remove them from their lobby...
        let mut event = 0xFFFF;
        {
            let lsr = self.lobbies.clone();
            let mut lobbies = lsr.borrow_mut();
            for l in lobbies.iter_mut() {
                if l.has_player(self.client_id) {
                    let cid = self.client_id;
                    event = l.event_num();
                    l.remove_player(self, cid).unwrap();
                    break;
                }
            }
            if event == 0xFFFF {
                // player isn't in a lobby...
                self.send_fatal_error(self.client_id, "\tEIllegal message");
                return
            }
        }

        // create the party
        let pass: Option<&str> = if m.password.len() == 0 { None } else { Some(&m.password) };
        let mut p = Party::new(&m.name, pass, m.episode, m.difficulty, m.battle != 0, m.challenge != 0, m.single_player != 0, event, self.online_maps.clone());

        let cid = self.client_id;
        p.add_player(self, cid).unwrap();

        parties.push(p);
    }

    pub fn bb_subcmd_60(&mut self, m: BbSubCmd60) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_60(self, m).unwrap();
                    return
                }
            }
        }
        {
            let pr = self.parties.clone();
            let ref mut parties = pr.borrow_mut();
            for p in parties.iter_mut() {
                let cid = self.client_id;
                if p.has_player(cid) {
                    p.handle_bb_subcmd_60(self, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_subcmd_62(&mut self, m: BbSubCmd62) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_62(self, m).unwrap();
                    return
                }
            }
        }
        {
            let pr = self.parties.clone();
            let ref mut parties = pr.borrow_mut();
            for p in parties.iter_mut() {
                let cid = self.client_id;
                if p.has_player(cid) {
                    p.handle_bb_subcmd_62(self, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_subcmd_6c(&mut self, m: BbSubCmd6C) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_6c(self, m).unwrap();
                    return
                }
            }
        }
        {
            let pr = self.parties.clone();
            let ref mut parties = pr.borrow_mut();
            for p in parties.iter_mut() {
                let cid = self.client_id;
                if p.has_player(cid) {
                    p.handle_bb_subcmd_6c(self, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_subcmd_6d(&mut self, m: BbSubCmd6D) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_6d(self, m).unwrap();
                    return
                }
            }
        }
        {
            let pr = self.parties.clone();
            let ref mut parties = pr.borrow_mut();
            for p in parties.iter_mut() {
                let cid = self.client_id;
                if p.has_player(cid) {
                    p.handle_bb_subcmd_6d(self, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_lobby_change(&mut self, m: LobbyChange) {
        let lr = self.lobbies.clone();
        let ref mut lobbies = lr.borrow_mut();
        // first, check if that lobby isn't full
        match m.1 {
            l @ 1 ... 15 => {
                if lobbies[l as usize-1].is_full() {
                    self.send_error(self.client_id, "\tELobby is full.");
                    return
                }
            },
            _ => {
                warn!("Client {} tried to join an invalid lobby", self.client_id);
                self.send_fatal_error(self.client_id, "\tEYou tried to join an invalid lobby.\nDisconnected.");
                return
            }
        }
        let cid = self.client_id;
        for l in lobbies.iter_mut() {
            if l.has_player(cid) {
                l.remove_player(self, cid).unwrap();
                break
            }
        }
        lobbies[m.1 as usize-1].add_player(self, cid).unwrap();
    }

    pub fn bb_game_name(&mut self) {
        let pr = self.parties.clone();
        let ref mut parties = pr.borrow_mut();
        for p in parties.iter_mut() {
            if p.has_player(self.client_id) {
                p.handle_bb_game_name(self).unwrap();
                return
            }
        }
        warn!("Client {} requested party name when they weren't in a party.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message.");
        return
    }

    pub fn bb_game_list(&mut self) {
        let mut games = Vec::new();
        let pr = self.parties.clone();
        let ref mut parties = pr.borrow_mut();

        let mut game = BbGameListEntry::default();
        game.menu_id = MENU_GAME_LIST;
        game.item_id = 0xFFFFFFFF;
        game.flags = 0x4;
        games.push(game);

        for (i, p) in parties.iter().enumerate() {
            let mut game = BbGameListEntry::default();
            game.menu_id = MENU_GAME_LIST;
            game.item_id = i as u32;
            game.name = p.name.clone();
            game.episode = (4 << 4) | p.episode;
            game.difficulty = 0x22 + p.difficulty;
            game.players = p.num_players() as u8;
            if p.password.is_some() {
                game.flags |= 0x2;
            }
            if p.battle {
                game.flags |= 0x10;
            }
            if p.challenge {
                game.flags |= 0x20;
            }
            if p.single_player {
                game.flags |= 0x4;
            }
            games.push(game);
        }
        let m: Message = Message::BbGameList(games.len() as u32 - 1, BbGameList { games: games });
        self.send_to_client(self.client_id, m);
    }

    pub fn bb_player_leave_game(&mut self, _m: BbPlayerLeaveGame) {
        let pr = self.parties.clone();
        let ref mut parties = pr.borrow_mut();
        let mut removed = 0;
        let mut party_index = 0;
        for (i, p) in parties.iter_mut().enumerate() {
            if p.has_player(self.client_id) {
                let cid = self.client_id;
                if p.remove_player(self, cid).unwrap() {
                    // remove the party
                    party_index = i;
                    removed = 2;
                    break;
                }
                removed = 1;
                break;
            }
        }
        match removed {
            1 => {
                // party will live on
            },
            2 => {
                // party is empty; remove now
                parties.remove(party_index);
            },
            _ => {
                warn!("Client {} was not in a game and tried to leave one", self.client_id);
                self.send_fatal_error(self.client_id, "\tEIllegal message");
            }
        }

    }
}

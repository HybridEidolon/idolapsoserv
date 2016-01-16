use std::rc::Rc;
use std::sync::Arc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
//use std::fs::File;

use mio::Sender;

use psomsg::bb::*;

use psodata::battleparam::BattleParamTables;
use psodata::leveltable::LevelTable;

//use ::game::CharClass;
use ::shipgate::client::callbacks::SgCbMgr;
use ::loop_handler::LoopMsg;
use ::shipgate::msg::Message as Sgm;
use ::shipgate::msg::BbLoginChallenge;
use ::shipgate::msg::BbGetAccountInfo;
use ::shipgate::msg::BbGetCharacter;
use ::shipgate::msg::BbGetCharacterAck;
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
    online_maps: Arc<Areas>,
    offline_maps: Arc<Areas>,
    pub level_table: Arc<LevelTable>,
    party_counter: Rc<Cell<u32>>
}

impl BlockHandler {
    pub fn new(sender: Sender<LoopMsg>,
               sg_sender: SgCbMgr<BlockHandler>,
               client_id: usize,
               clients: Rc<RefCell<HashMap<usize, Rc<RefCell<ClientState>>>>>,
               lobbies: Rc<RefCell<Vec<Lobby>>>,
               parties: Rc<RefCell<Vec<Party>>>,
               battle_params: Arc<BattleParamTables>,
               online_maps: Arc<Areas>,
               offline_maps: Arc<Areas>,
               level_table: Arc<LevelTable>,
               party_counter: Rc<Cell<u32>>) -> BlockHandler {
        BlockHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id,
            clients: clients,
            lobbies: lobbies,
            parties: parties,
            battle_params: battle_params,
            online_maps: online_maps,
            offline_maps: offline_maps,
            level_table: level_table,
            party_counter: party_counter
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
                h.sg_sender.request(h.client_id, sgm, move|mut h, m| {
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
                        c.account_id = a.account_id;

                        // We need to get their character now.
                        let sgm: Sgm = BbGetCharacter { account_id: a.account_id, slot: sec_data.slot }.into();
                        h.sg_sender.request(h.client_id, sgm, move |mut h, m| {
                            if let Sgm::BbGetCharacterAck(_, body) = m {
                                h.sg_get_character_ack(body)
                            }
                        }).unwrap();
                    }
                }).unwrap();
            } else {
                warn!("Unexpected response from shipgate: {:?}", m);
                h.sender.send(LoopMsg::DropClient(h.client_id)).unwrap();
                return
            }
        }).unwrap();
    }

    fn sg_get_character_ack(&mut self, m: BbGetCharacterAck) {
        if m.status != 0 {
            error!("Shipgate error retrieving character, status code {}", m.status);
            self.send_fatal_error(self.client_id, "Shipgate error retrieving character");
            return
        }
        if m.full_char.is_none() {
            // Somehow they got here and the shipgate doesn't have a char.
            error!("Illegal state. Character in slot is missing.");
            self.send_fatal_error(self.client_id, "Illegal state. Character in slot is missing.");
            return
        }
        let BbGetCharacterAck { full_char, .. } = m;
        let full_char = full_char.unwrap();

        let cs = self.get_client_state(self.client_id).unwrap();
        let mut client_state = cs.borrow_mut();
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
                self.sender.send((self.client_id, r).into()).unwrap();
            }

            let r = Message::BbFullChar(0, BbFullChar(full_char.clone()));
            self.sender.send((self.client_id, r).into()).unwrap();
            client_state.full_char = Some(full_char);
            let r = Message::CharDataRequest(0, CharDataRequest);
            self.sender.send((self.client_id, r).into()).unwrap();
        }
        return
    }

    fn get_new_party_id(&mut self) -> u32 {
        let pcr = self.party_counter.clone();
        let ret = pcr.get();
        pcr.set(ret + 1);
        ret
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
        self.send_fatal_error(self.client_id, "\tEAll lobbies on this block are full. Please try again.");
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
                    info!("<{:02}-{:02}> {}: {}", l.block_num(), l.lobby_num() + 1, player_name.trim_left_matches("\tE"), m.1.trim_left_matches("\tE"));
                    m.0 = gc_num;
                    l.bb_broadcast(self, None, m.into()).unwrap();
                    return
                }
            }
        }
        // Then, check if they're in a party
        {
            let pr = self.parties.clone();
            let ref mut parties = pr.borrow_mut();
            for p in parties.iter_mut() {
                if p.has_player(self.client_id) {
                    let cid = self.client_id;
                    info!("<{}> {}: {}", &p.name[2..], player_name.trim_left_matches("\tE"), m.1.trim_left_matches("\tE"));
                    m.0 = gc_num;
                    p.handle_chat(self, cid, &m.1).unwrap();
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
        let unique_id = self.get_new_party_id();
        let pass: Option<&str> = if m.password.len() == 0 { None } else { Some(&m.password) };
        let mut p;
        if m.single_player > 0 {
            p = Party::new(&m.name, pass, m.episode, m.difficulty, m.battle != 0, m.challenge != 0, true, event, self.offline_maps.clone(), unique_id);
        } else {
            p = Party::new(&m.name, pass, m.episode, m.difficulty, m.battle != 0, m.challenge != 0, false, event, self.online_maps.clone(), unique_id);
        }

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
                    p.handle_bb_subcmd_60(self, cid, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_subcmd_62(&mut self, dest: u32, m: BbSubCmd62) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_62(self, dest, m).unwrap();
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
                    p.handle_bb_subcmd_62(self, cid, dest, m).unwrap();
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
                    p.handle_bb_subcmd_6c(self, cid, m).unwrap();
                    return
                }
            }
        }

        warn!("Could not find the lobby or party this subcmd was sent by {}.", self.client_id);
        self.send_fatal_error(self.client_id, "\tEIllegal message");
    }

    pub fn bb_subcmd_6d(&mut self, dest: u32, m: BbSubCmd6D) {
        {
            let lr = self.lobbies.clone();
            let ref mut lobbies = lr.borrow_mut();
            for l in lobbies.iter_mut() {
                let cid = self.client_id;
                if l.has_player(cid) {
                    l.handle_bb_subcmd_6d(self, dest, m).unwrap();
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
                    p.handle_bb_subcmd_6d(self, cid, dest, m).unwrap();
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

        for p in parties.iter(){
            let mut game = BbGameListEntry::default();
            game.menu_id = MENU_GAME_LIST;
            game.item_id = p.unique_id as u32;
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

    pub fn bb_update_options(&mut self, m: BbUpdateOptions) {
        use ::shipgate::msg::BbUpdateOptions as SgBbUO;
        info!("{} updated general options", self.client_id);
        let options = m.0;
        let cr = self.get_client_state(self.client_id).unwrap();
        let ref client_state = cr.borrow();
        self.sg_sender.send(Sgm::BbUpdateOptions(0, SgBbUO {
            account_id: client_state.account_id,
            options: options
        })).unwrap();
    }

    pub fn bb_update_keys(&mut self, m: BbUpdateKeys) {
        use ::shipgate::msg::BbUpdateKeys as SgBbUK;
        info!("{} updated keyboard configuration", self.client_id);
        let keys = m.0;
        let cr = self.get_client_state(self.client_id).unwrap();
        let ref client_state = cr.borrow();
        self.sg_sender.send(Sgm::BbUpdateKeys(0, SgBbUK {
            account_id: client_state.account_id,
            key_config: keys
        })).unwrap();
    }

    pub fn bb_update_joy(&mut self, m: BbUpdateJoy) {
        use ::shipgate::msg::BbUpdateJoy as SgBbJ;
        info!("{} updated joystick configuration", self.client_id);
        let joy = m.0;
        let cr = self.get_client_state(self.client_id).unwrap();
        let ref client_state = cr.borrow();
        self.sg_sender.send(Sgm::BbUpdateJoy(0, SgBbJ {
            account_id: client_state.account_id,
            joy_config: joy
        })).unwrap();
    }

    pub fn menu_select(&mut self, m: MenuSelect) {
        let MenuSelect(menu_id, item_id) = m;
        match menu_id {
            MENU_GAME_LIST => {
                let pr = self.parties.clone();
                let mut parties = pr.borrow_mut();
                for p in parties.iter_mut() {
                    if p.unique_id == item_id {
                        let cid = self.client_id;
                        // First, verify they can join the game
                        if p.is_bursting() {
                            self.send_error(self.client_id, "\tEA player is bursting.\nPlease wait.");
                            return
                        }
                        if p.player_limit() == 1 {
                            self.send_error(self.client_id, "\tEParty is One Person only.");
                            return
                        }
                        if p.is_full() {
                            self.send_error(self.client_id, "\tEParty is full.");
                            return
                        }

                        // Then, remove them from their lobby
                        let lr = self.lobbies.clone();
                        let mut lobbies = lr.borrow_mut();
                        for l in lobbies.iter_mut() {
                            if l.has_player(cid) {
                                if let Err(e) = l.remove_player(self, cid) {
                                    error!("Failed to remove player from lobby after party join: {:?}", e);
                                    self.send_fatal_error(self.client_id, &format!("\tE{:?}", e));
                                    return
                                }
                                break
                            }
                        }
                        // Then add them to their game
                        if let Err(e) = p.add_player(self, cid) {
                            error!("Failed to join party: {:?}", e);
                            self.send_fatal_error(self.client_id, &format!("\tE{:?}", e));
                            return
                        }
                        break
                    }
                }
                self.send_error(self.client_id, "\tEParty no longer\texists.");
            },
            _ => {
                self.send_error(self.client_id, "\tEInvalid menu");
                return
            }
        }
    }

    pub fn done_burst(&mut self) {
        let pr = self.parties.clone();
        let mut parties = pr.borrow_mut();
        for p in parties.iter_mut() {
            if p.has_player(self.client_id) {
                p.handle_bb_done_burst(self).unwrap();
                break
            }
        }
    }

    pub fn bb_full_char(&mut self, m: BbFullChar) {
        // TODO verify... or just track based on their other messages sent
        // this is prone to being cheated. we'll just save some parts until
        // the implementation is more thorough.
        let BbFullChar(full_char) = m;

        let BbFullCharData { inv, chara, bank, .. } = full_char;

        let cs = self.get_client_state(self.client_id).unwrap();
        let ref mut client_state = cs.borrow_mut();
        if let Some(ref mut cur_fc) = client_state.full_char {
            info!("Client {} triggered manual save", self.client_id);
            cur_fc.inv = inv;
            cur_fc.chara = chara;
            cur_fc.bank = bank;
        } else {
            warn!("Client sent full character but we didn't have one loaded for them. This is an abnormal state.");
            return
        }
    }
}

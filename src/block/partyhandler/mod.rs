//! Where the majority of the game occurs.

use std::sync::Arc;
use std::collections::VecDeque;

pub mod error;
pub mod enemygen;

use rand::random;

use psomsg::bb::Message as BbMsg;
use psomsg::bb::*;

use psodata::map::MapEnemy;

use ::maps::{Areas, InstanceEnemy, Ep1Areas, Ep2Areas, Ep4Areas};

use super::handler::BlockHandler;

use self::error::PartyError;
use self::enemygen::convert_enemy;

#[derive(Clone, Debug)]
pub struct Party {
    pub name: String,
    pub password: Option<String>,
    pub episode: u8,
    pub difficulty: u8,
    pub battle: bool,
    pub challenge: bool,
    pub single_player: bool,
    pub unique_id: u32,
    section_id: Option<u8>,
    members: [Option<usize>; 4],
    bursting: [bool; 4],
    leader_id: u8,
    maps: Arc<Areas>,
    variants: Vec<u32>,
    enemies: Vec<InstanceEnemy>,
    bc_queue: VecDeque<(usize, Message)>
}

impl Party {
    pub fn new(name: &str, password: Option<&str>, episode: u8, difficulty: u8, battle: bool, challenge: bool, single_player: bool, event: u16, maps: Arc<Areas>, unique_id: u32) -> Party {
        // pick random variants for each map based on episode
        let (variants, enemies) = match episode {
            1 => {
                info!("Generating episode 1 party");
                Party::random_variants_ep1(&maps.ep1, event, difficulty > 0)
            },
            2 => {
                info!("Generating episode 2 party");
                Party::random_variants_ep2(&maps.ep2, event)
            },
            3 => {
                info!("Generating episode 4 party");
                Party::random_variants_ep4(&maps.ep4, event)
            },
            _ => panic!("unsupported episode")
        };
        info!("{} total enemies", enemies.len());
        Party {
            name: name.to_owned(),
            password: password.map(|s| s.to_owned()),
            episode: episode,
            difficulty: difficulty,
            battle: battle,
            challenge: challenge,
            single_player: single_player,
            unique_id: unique_id,
            section_id: None,
            members: Default::default(),
            bursting: Default::default(),
            bc_queue: Default::default(),
            leader_id: 0,
            maps: maps,
            variants: variants,
            enemies: enemies
        }
    }

    /// Broadcasts a Blue Burst message to all players in the lobby. Performs
    /// conversion on the message to the appropriate client versions if
    /// necessary. If sent_by is `Some`, that client will not receive the
    /// message.
    pub fn bb_broadcast(&self, handler: &mut BlockHandler, sent_by: Option<usize>, msg: BbMsg) -> Result<(), PartyError> {
        for co in self.members.iter() {
            match co {
                &Some(c) => {
                    if (sent_by.is_some() && c != sent_by.unwrap()) || sent_by.is_none() {
                        handler.send_to_client(c, msg.clone());
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }

    pub fn add_player(&mut self, handler: &mut BlockHandler, player: usize) -> Result<(), PartyError> {
        info!("Adding client {} to party \"{}\"", player, &self.name[2..]);

        let new_client_id = match self.find_first_empty() {
            Some(slot) => slot,
            None => return Err(PartyError::IsFull)
        };
        if self.num_players() == 0 {
            info!("Initial leader for \"{}\" set", &self.name[2..]);
            self.leader_id = new_client_id;
        }

        // put them in that slot
        self.members[new_client_id as usize] = Some(player);

        info!("New client ID is {}", new_client_id);

        let mut l = BbGameJoin::default();
        l.maps = self.variants.clone();
        l.client_id = new_client_id;
        l.leader_id = self.leader_id;
        l.one = 1;
        l.one2 = 1;
        l.difficulty = self.difficulty;
        l.episode = self.episode;
        if let Some(sid) = self.section_id {
            l.section = sid;
        } else {
            // first joiner's section id is the one for the party
            let cr = handler.get_client_state(player).unwrap();
            let c = cr.borrow();
            self.section_id = Some(c.full_char.as_ref().unwrap().chara.section);
            l.section = c.full_char.as_ref().unwrap().section;
        }
        l.single_player = if self.single_player {1} else {0};
        for (i, po) in self.members.iter().enumerate() {
            match po {
                &Some(cid) => {
                    let cr = handler.get_client_state(cid).unwrap();
                    let c = cr.borrow();
                    let mut ph = PlayerHdr::default();
                    ph.tag = 0x00010000;
                    ph.guildcard = c.bb_guildcard;
                    ph.client_id = i as u32;
                    ph.name = c.full_char.as_ref().unwrap().chara.name.clone();
                    l.players.push(ph);
                },
                _ => {
                    l.players.push(PlayerHdr::default());
                }
            }
        }
        let r: Message = Message::BbGameJoin(self.num_players() as u32, l);
        handler.send_to_client(player, r);

        self.bursting[new_client_id as usize] = true;

        // tell the other clients about the player
        for (i, po) in self.members.iter().enumerate() {
            match po {
                &Some(cid) => {
                    let cr = handler.get_client_state(player).unwrap();
                    let c = cr.borrow();
                    let mut l = BbGameAddMember::default();
                    let mut ph = LobbyMember::default();
                    l.one = 1;
                    l.leader_id = self.leader_id;
                    l.client_id = i as u8;
                    l.lobby_num = 0xFF;
                    l.block_num = 1;
                    l.event = 1;
                    ph.hdr.tag = 0x00010000;
                    ph.hdr.guildcard = c.bb_guildcard;
                    ph.hdr.client_id = new_client_id as u32;
                    ph.hdr.name = c.full_char.as_ref().unwrap().chara.name.clone();
                    ph.inventory = c.full_char.as_ref().unwrap().inv.clone();
                    ph.data = c.full_char.as_ref().unwrap().chara.clone();
                    l.member = ph;
                    let m: Message = Message::BbGameAddMember(1, l);
                    handler.send_to_client(cid, m);
                },
                _ => ()
            }
        }

        info!("{:?}", self.members);

        Ok(())
    }

    pub fn remove_player(&mut self, handler: &mut BlockHandler, player: usize) -> Result<bool, PartyError> {
        let mut ret = false;
        match self.client_id_for_player(player) {
            Some(i) => {
                info!("Removing client {} from party \"{}\"", player, &self.name[2..]);
                if self.leader_id == i {
                    // pick a new leader
                    match self.find_first_player_not_matching(player) {
                        Some((ii, _)) => {
                            info!("New leader for \"{}\" elected to {}", &self.name[2..], ii);
                            self.leader_id = ii;
                        },
                        None => {
                            info!("Party {} is being removed", self.name);
                            // the party is empty, we'll return true to destroy the party
                            ret = true;
                        }
                    }
                }
                self.members[i as usize] = None;
                // ensure their bursting flag is unset
                self.bursting[i as usize] = false;

                // tell the other clients that this player has left, and maybe
                // the new elected leader
                // if the party is empty, this will do nothing
                let gl = BbGameLeave {
                    client_id: i,
                    leader_id: self.leader_id,
                    padding: 0
                };
                self.bb_broadcast(handler, Some(player), gl.into()).unwrap();
                info!("{:?}", self.members);
            },
            _ => {
                return Err(PartyError::NotInParty)
            }
        }

        Ok(ret)
    }

    pub fn handle_bb_game_name(&mut self, handler: &mut BlockHandler) -> Result<(), PartyError> {
        handler.send_to_client(handler.client_id, BbGameName(self.name.clone()).into());
        Ok(())
    }

    pub fn num_players(&self) -> usize {
        let mut count = 0;
        for mo in self.members.iter() {
            match mo {
                &Some(_) => {
                    count += 1;
                },
                _ => ()
            }
        }
        count
    }

    pub fn has_player(&self, player: usize) -> bool {
        for mo in self.members.iter() {
            match mo {
                &Some(m) if m == player => {
                    return true
                },
                _ => ()
            }
        }
        false
    }

    pub fn is_full(&self) -> bool {
        self.num_players() >= 4
    }

    pub fn is_empty(&self) -> bool {
        self.num_players() == 0
    }

    /// If any player is bursting, returns true.
    pub fn is_bursting(&self) -> bool {
        for b in self.bursting.iter() {
            if *b {
                return true
            }
        }
        false
    }

    /// The player limit of this party. If this is single player, it's 1, else 4
    pub fn player_limit(&self) -> usize {
        if self.single_player {
            1
        } else {
            4
        }
    }

    pub fn handle_bb_done_burst(&mut self, handler: &mut BlockHandler) -> Result<(), PartyError> {
        // We tell party members with a subcmd, not the main cmd...
        if let Some(local_slot) = self.client_id_for_player(handler.client_id) {
            if self.bursting[local_slot as usize] {
                // send a ping real quick
                handler.send_to_client(handler.client_id, Message::Ping(0, Ping));
                self.bursting[local_slot as usize] = false;
                let mut burst = Bb60DoneBurst::default();
                burst.data[0] = 0x18;
                burst.data[1] = 0x08;
                self.bb_broadcast(handler, None, Message::BbSubCmd60(0, BbSubCmd60::Bb60DoneBurst { client_id: 0, unused: 0, data: burst })).unwrap();
            } else {
                return Err(PartyError::NotBursting)
            }
        } else {
            return Err(PartyError::NotInParty)
        }

        // empty the message queue if we're no longer bursting
        // only one player can burst at a time so this should always execute
        if !self.is_bursting() {
            loop {
                if let Some((sender, m)) = self.bc_queue.pop_front() {
                    match m {
                        Message::BbSubCmd60(_, msg) => {
                            if let Err(e) = self.handle_bb_subcmd_60(handler, sender, msg) {
                                return Err(e)
                            }
                        },
                        Message::BbSubCmd6C(_, msg) => {
                            if let Err(e) = self.handle_bb_subcmd_6c(handler, sender, msg) {
                                return Err(e)
                            }
                        },
                        Message::BbSubCmd62(dest, msg) => {
                            if let Err(e) = self.handle_bb_subcmd_62(handler, sender, dest, msg) {
                                return Err(e)
                            }
                        },
                        Message::BbSubCmd6D(dest, msg) => {
                            if let Err(e) = self.handle_bb_subcmd_6d(handler, sender, dest, msg) {
                                return Err(e)
                            }
                        },
                        _ => {
                            unreachable!()
                        }
                    }
                } else {
                    // queue is empty
                    break
                }
            }
        } else {
            unreachable!()
        }
        Ok(())
    }

    pub fn handle_bb_subcmd_60(&mut self, handler: &mut BlockHandler, sender: usize, m: BbSubCmd60) -> Result<(), PartyError> {
        if self.is_bursting() {
            if let &BbSubCmd60::Unknown { cmd, .. } = &m {
                if cmd == 0x7C {
                    // safe to send during burst, ignore
                } else {
                    // enqueue
                    self.bc_queue.push_back((sender, Message::BbSubCmd60(0, m)));
                    return Ok(())
                }
            }
        }
        match m.clone() {
            BbSubCmd60::Bb60ReqExp { data: r, .. } => {
                self.handle_bb_60_req_exp(handler, r);
            },
            _ => ()
        }
        debug!("{} bc 0x60: {:?}", sender, m);
        self.bb_broadcast(handler, Some(sender), m.into())
    }

    pub fn handle_bb_subcmd_62(&mut self, handler: &mut BlockHandler, sender: usize, dest: u32, m: BbSubCmd62) -> Result<(), PartyError> {
        if self.is_bursting() {
            // queue all but some messages
            match &m {
                &BbSubCmd62::Unknown { cmd, .. } => {
                    match cmd {
                        0x6B | 0x6C | 0x6D | 0x6E | 0x6F | 0x70 | 0x71 => {
                            // these are safe to send during bursting
                            // do nothing; fall to actual handle step
                        },
                        _ => {
                            // enqueue if bursting
                            self.bc_queue.push_back((sender, Message::BbSubCmd62(dest, m.clone())));
                            return Ok(())
                        }
                    }
                },
                m => {
                    self.bc_queue.push_back((sender, Message::BbSubCmd62(dest, m.clone())));
                    return Ok(())
                }
            }
        }
        if let Some(dest_cid) = self.members[dest as usize] {
            handler.send_to_client(dest_cid, Message::BbSubCmd62(dest, m.clone()));
        } else {
            // ignore
            return Ok(())
        }
        debug!("{} bc 0x62 dest {}: {:?}", sender, dest, m);
        Ok(())
    }

    pub fn handle_bb_subcmd_6c(&mut self, handler: &mut BlockHandler, sender: usize, m: BbSubCmd6C) -> Result<(), PartyError> {
        if self.is_bursting() {
            if let &BbSubCmd6C::Unknown { cmd, .. } = &m {
                if cmd == 0x7C {
                    // safe to send during burst, ignore
                } else {
                    // enqueue
                    self.bc_queue.push_back((sender, Message::BbSubCmd6C(0, m)));
                    return Ok(())
                }
            }
        }
        match m.clone() {
            BbSubCmd6C::Bb60ReqExp { data: r, .. } => {
                self.handle_bb_60_req_exp(handler, r);
            },
            _ => ()
        }
        debug!("{} bc 0x6c: {:?}", sender, m);
        self.bb_broadcast(handler, Some(sender), m.into())
    }

    pub fn handle_bb_subcmd_6d(&mut self, handler: &mut BlockHandler, sender: usize, dest: u32, m: BbSubCmd6D) -> Result<(), PartyError> {
        if self.is_bursting() {
            // queue all but some messages
            match &m {
                &BbSubCmd6D::Unknown { cmd, .. } => {
                    match cmd {
                        0x6B | 0x6C | 0x6D | 0x6E | 0x6F | 0x70 | 0x71 => {
                            // these are safe to send during bursting
                            // do nothing; fall to actual handle step
                        },
                        _ => {
                            // enqueue if bursting
                            self.bc_queue.push_back((sender, Message::BbSubCmd6D(dest, m.clone())));
                            return Ok(())
                        }
                    }
                },
            }
        }
        if let Some(&Some(dest_cid)) = self.members.get(dest as usize) {
            handler.send_to_client(dest_cid, Message::BbSubCmd6D(dest, m.clone()));
        } else {
            // ignore
            return Ok(())
        }
        debug!("{} bc 0x6d dest {}: {:?}", sender, dest, m);
        Ok(())
    }

    pub fn handle_bb_60_req_exp(&mut self, handler: &mut BlockHandler, m: Bb60ReqExp) {
        let cid = handler.client_id;
        debug!("Client {} request exp: {:?}", cid, m);
        if let Some(ref enemy) = self.enemies.get(m.enemy_id as usize) {
            let bp = {
                match self.episode {
                    1 => handler.battle_params.get_ep1(enemy.param_entry, self.single_player, self.difficulty).cloned(),
                    2 => handler.battle_params.get_ep2(enemy.param_entry, self.single_player, self.difficulty).cloned(),
                    3 => handler.battle_params.get_ep4(enemy.param_entry, self.single_player, self.difficulty).cloned(),
                    _ => unreachable!()
                }
            };

            if let Some(bp) = bp {
                if m.last_hitter == 1 {
                    let exp = bp.exp;
                    info!("Client {} request verified; +{} EXP for last-hitting on {} ({})", cid, exp, enemy.name, m.enemy_id);
                    self.award_exp(cid, handler, exp);
                } else {
                    let exp = bp.exp * 80 / 100;
                    info!("Client {} request verified; +{} EXP for assisting on {} ({})", cid, exp, enemy.name, m.enemy_id);
                    self.award_exp(cid, handler, exp);
                }

            } else {
                error!("Battle param entry for enemy id {} doesn't exist", m.enemy_id);
                return
            }
        } else {
            warn!("Client {} tried to gain exp for an enemy that doesn't exist: {}", cid, m.enemy_id);
            return
        }
    }

    fn award_exp(&self, client: usize, handler: &mut BlockHandler, exp: u32) {
        let mut leveled_up = false;
        let mut current_level;
        let mut stats = Default::default();
        {
            let cr = handler.get_client_state(client).unwrap();
            let ref mut client_state = cr.borrow_mut();
            let lt = handler.level_table.clone();
            let mut chara = client_state.full_char.as_mut().unwrap();
            current_level = chara.chara.level as usize;
            let current_exp = chara.chara.exp as usize;

            if current_level >= 199 {
                // Can't gain any more experience.
                return
            }


            loop {
                if current_level >= 199 {
                    // We hit 200 while adding levels, break
                    break
                }
                // Level table entry.
                let lte = lt.levels[chara.chara.class as usize][current_level + 1];
                debug!("Exp needed to level: {}, current: {}", lte.exp, current_exp);
                if current_exp + exp as usize >= lte.exp as usize {
                    info!("Client {} leveled up!", client);
                    leveled_up = true;

                    // Update their stats with award from new level.
                    chara.chara.stats.atp += lte.atp as u16;
                    chara.chara.stats.mst += lte.mst as u16;
                    chara.chara.stats.evp += lte.evp as u16;
                    chara.chara.stats.hp += lte.hp as u16;
                    chara.chara.stats.dfp += lte.dfp as u16;
                    chara.chara.stats.ata += lte.ata as u16;

                    // TODO add equipped mag bonuses

                    // Update their level.
                    current_level += 1;

                    stats = chara.chara.stats;
                } else {
                    break
                }
            }

            chara.chara.exp += exp;
            chara.chara.level = current_level as u32;
        }
        let slot = self.client_id_for_player(client).unwrap();

        self.bb_broadcast(handler, None, Message::BbSubCmd60(0, BbSubCmd60::Bb60GiveExp { client_id: slot, unused: 0, data: Bb60GiveExp(exp) })).unwrap();

        if leveled_up {
            self.bb_broadcast(handler, None, Message::BbSubCmd60(0, BbSubCmd60::Bb60LevelUp { client_id: slot, unused: 0, data: Bb60LevelUp {
                atp: stats.atp,
                mst: stats.mst,
                evp: stats.evp,
                hp: stats.hp,
                dfp: stats.dfp,
                ata: stats.ata,
                level: current_level as u32
            }})).unwrap();
        }
    }

    fn find_first_player_not_matching(&self, player: usize) -> Option<(u8, usize)> {
        for (i, po) in self.members.iter().enumerate() {
            match po {
                &Some(cid) if cid != player => {
                    return Some((i as u8, cid))
                },
                _ => continue
            }
        }
        None
    }

    // fn find_first_player(&self) -> Option<(u8, usize)> {
    //     for (i, po) in self.members.iter().enumerate() {
    //         match po {
    //             &Some(cid) => {
    //                 return Some((i as u8, cid))
    //             },
    //             _ => continue
    //         }
    //     }
    //     None
    // }

    fn find_first_empty(&self) -> Option<u8> {
        for (i, po) in self.members.iter().enumerate() {
            match po {
                &None => {
                    return Some(i as u8)
                },
                _ => continue
            }
        }
        None
    }

    fn client_id_for_player(&self, player: usize) -> Option<u8> {
        for (i, mo) in self.members.iter().enumerate() {
            match mo {
                &Some(m) if m == player => {
                    return Some(i as u8)
                },
                _ => ()
            }
        }
        None
    }

    fn random_variants_ep1(maps: &Ep1Areas, event: u16, normal: bool) -> (Vec<u32>, Vec<InstanceEnemy>) {
        let mut variants = vec![0; 0x20];
        let mut enemies = Vec::with_capacity(0xB50);
        Party::append_enemies(&maps.city.enemies, &mut enemies, 1, event, false);

        // Forest
        variants[3] = (random::<usize>() % maps.forest1.len()) as u32;
        variants[5] = (random::<usize>() % maps.forest2.len()) as u32;
        Party::append_enemies(&maps.forest1[variants[3] as usize].enemies, &mut enemies, 1, event, false);
        Party::append_enemies(&maps.forest2[variants[5] as usize].enemies, &mut enemies, 1, event, false);

        {
            let keys: Vec<_> = maps.cave1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[6] = m;
            variants[7] = v;
            Party::append_enemies(&maps.cave1.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.cave2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[8] = m;
            variants[9] = v;
            Party::append_enemies(&maps.cave2.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.cave3.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[10] = m;
            variants[11] = v;
            Party::append_enemies(&maps.cave3.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.machine1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[12] = m;
            variants[13] = v;
            Party::append_enemies(&maps.machine1.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.machine2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[14] = m;
            variants[15] = v;
            Party::append_enemies(&maps.machine2.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.ancient1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[16] = m;
            variants[17] = v;
            Party::append_enemies(&maps.ancient1.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.ancient2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[18] = m;
            variants[19] = v;
            Party::append_enemies(&maps.ancient2.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
            let keys: Vec<_> = maps.ancient3.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[20] = m;
            variants[21] = v;
            Party::append_enemies(&maps.ancient3.get(&(m, v)).unwrap().enemies, &mut enemies, 1, event, false);
        }
        Party::append_enemies(&maps.boss1.enemies, &mut enemies, 1, event, false);
        Party::append_enemies(&maps.boss2.enemies, &mut enemies, 1, event, false);
        Party::append_enemies(&maps.boss3.enemies, &mut enemies, 1, event, false);
        Party::append_enemies(&maps.boss4.enemies, &mut enemies, 1, event, false);

        if !normal {
            // Dark Falz has a different param entry on Hard+ because of third phase
            for e in enemies.iter_mut().filter(|e| e.param_entry == 0x37) {
                e.param_entry = 0x38;
            }
        }

        (variants, enemies)
    }

    fn random_variants_ep2(maps: &Ep2Areas, event: u16) -> (Vec<u32>, Vec<InstanceEnemy>) {
        let mut variants = vec![0; 0x20];
        let mut enemies = Vec::new();
        Party::append_enemies(&maps.city.enemies, &mut enemies, 2, event, false);

        {
            let keys: Vec<_> = maps.ruins1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[2] = m;
            variants[3] = v;
            Party::append_enemies(&maps.ruins1.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);
            let keys: Vec<_> = maps.ruins2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[4] = m;
            variants[5] = v;
            Party::append_enemies(&maps.ruins2.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);
            let keys: Vec<_> = maps.space1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[6] = m;
            variants[7] = v;
            Party::append_enemies(&maps.space1.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);
            let keys: Vec<_> = maps.space2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[8] = m;
            variants[9] = v;
            Party::append_enemies(&maps.space2.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);

            variants[11] = (random::<usize>() % maps.jungle1.len()) as u32;
            variants[13] = (random::<usize>() % maps.jungle2.len()) as u32;
            variants[15] = (random::<usize>() % maps.jungle3.len()) as u32;
            variants[19] = (random::<usize>() % maps.jungle5.len()) as u32;
            Party::append_enemies(&maps.jungle1[variants[11] as usize].enemies, &mut enemies, 2, event, false);
            Party::append_enemies(&maps.jungle2[variants[13] as usize].enemies, &mut enemies, 2, event, false);
            Party::append_enemies(&maps.jungle3[variants[15] as usize].enemies, &mut enemies, 2, event, false);

            let keys: Vec<_> = maps.jungle4.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[16] = m;
            variants[17] = v;
            Party::append_enemies(&maps.jungle4.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);

            Party::append_enemies(&maps.jungle5[variants[19] as usize].enemies, &mut enemies, 2, event, false);

            let keys: Vec<_> = maps.seabed1.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[20] = m;
            variants[21] = v;
            Party::append_enemies(&maps.seabed1.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);
            let keys: Vec<_> = maps.seabed2.keys().collect();
            let i = random::<usize>() % keys.len();
            let &(m, v) = keys[i];
            variants[22] = m;
            variants[23] = v;
            Party::append_enemies(&maps.seabed2.get(&(m, v)).unwrap().enemies, &mut enemies, 2, event, false);
        }
        // bosses
        Party::append_enemies(&maps.boss5.enemies, &mut enemies, 2, event, false);
        Party::append_enemies(&maps.boss6.enemies, &mut enemies, 2, event, false);
        Party::append_enemies(&maps.boss7.enemies, &mut enemies, 2, event, false);
        Party::append_enemies(&maps.boss8.enemies, &mut enemies, 2, event, false);

        (variants, enemies)
    }

    fn random_variants_ep4(maps: &Ep4Areas, event: u16) -> (Vec<u32>, Vec<InstanceEnemy>) {
        let mut variants = vec![0; 0x20];
        let mut enemies = Vec::new();
        Party::append_enemies(&maps.city.enemies, &mut enemies, 3, event, false);

        variants[3] = (random::<usize>() % maps.wilds1.len()) as u32;
        variants[5] = (random::<usize>() % maps.wilds2.len()) as u32;
        variants[7] = (random::<usize>() % maps.wilds3.len()) as u32;
        variants[9] = (random::<usize>() % maps.wilds4.len()) as u32;
        variants[11] = (random::<usize>() % maps.crater.len()) as u32;
        variants[12] = (random::<usize>() % maps.desert1.len()) as u32;
        variants[15] = (random::<usize>() % maps.desert2.len()) as u32;
        variants[16] = (random::<usize>() % maps.desert3.len()) as u32;

        Party::append_enemies(&maps.wilds1[variants[3] as usize].enemies, &mut enemies, 3, event, false);
        Party::append_enemies(&maps.wilds2[variants[5] as usize].enemies, &mut enemies, 3, event, false);
        Party::append_enemies(&maps.wilds3[variants[7] as usize].enemies, &mut enemies, 3, event, false);
        Party::append_enemies(&maps.wilds4[variants[9] as usize].enemies, &mut enemies, 3, event, false);
        Party::append_enemies(&maps.crater[variants[11] as usize].enemies, &mut enemies, 3, event, false);
        Party::append_enemies(&maps.desert1[variants[12] as usize].enemies, &mut enemies, 3, event, true);
        Party::append_enemies(&maps.desert2[variants[15] as usize].enemies, &mut enemies, 3, event, true);
        Party::append_enemies(&maps.desert3[variants[16] as usize].enemies, &mut enemies, 3, event, true);

        Party::append_enemies(&maps.boss9.enemies, &mut enemies, 3, event, false);

        (variants, enemies)
    }

    fn append_enemies(map_enemies: &[MapEnemy], instance_enemies: &mut Vec<InstanceEnemy>, episode: u8, event: u16, alt_enemies: bool) {
        for m in map_enemies.iter() {
            instance_enemies.append(&mut convert_enemy(m, episode, event, alt_enemies));
        }
    }
}

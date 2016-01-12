//! Where the majority of the game occurs.

use std::sync::Arc;

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
    members: [Option<usize>; 4],
    leader_id: u8,
    online_maps: Arc<Areas>,
    variants: Vec<u32>,
    enemies: Vec<InstanceEnemy>
}

impl Party {
    pub fn new(name: &str, password: Option<&str>, episode: u8, difficulty: u8, battle: bool, challenge: bool, single_player: bool, event: u16, online_maps: Arc<Areas>) -> Party {
        // pick random variants for each map based on episode
        let (variants, enemies) = match episode {
            1 => {
                info!("Generating episode 1 party");
                Party::random_variants_ep1(&online_maps.ep1, event)
            },
            2 => {
                info!("Generating episode 2 party");
                Party::random_variants_ep2(&online_maps.ep2, event)
            },
            3 => {
                info!("Generating episode 4 party");
                Party::random_variants_ep4(&online_maps.ep4, event)
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
            members: Default::default(),
            leader_id: 0,
            online_maps: online_maps,
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

        let cr = handler.get_client_state(player).unwrap();
        let c = cr.borrow();

        let mut l = BbGameJoin::default();
        l.maps = self.variants.clone();
        l.client_id = 0;
        l.leader_id = 0;
        l.one = 1;
        l.one2 = 1;
        l.difficulty = self.difficulty;
        l.episode = self.episode;
        let mut ph = PlayerHdr::default();
        ph.tag = 0x00010000;
        ph.guildcard = c.bb_guildcard;
        ph.client_id = 0;
        ph.name = c.full_char.as_ref().unwrap().name.clone();
        l.players.push(ph);
        let r: Message = l.into();
        handler.send_to_client(player, r);

        self.members[0] = Some(player);
        self.leader_id = 0;

        info!("{:?}", self.members);

        Ok(())
    }

    pub fn remove_player(&mut self, handler: &mut BlockHandler, player: usize) -> Result<bool, PartyError> {
        let mut ret = false;
        match self.client_id_for_player(player) {
            Some(i) => {
                info!("Removing client {} from party {}", player, self.name);
                if self.leader_id == i {
                    // pick a new leader
                    match self.find_first_player_not_matching(player) {
                        Some((ii, _)) => {
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

                // tell the other clients that this player has left, and maybe
                // the new elected leader
                // if the party is empty, this will do nothing
                let gl = BbGameLeave {
                    client_id: i,
                    leader_id: self.leader_id,
                    padding: 0
                };
                self.bb_broadcast(handler, Some(player), gl.into()).unwrap();
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
        !self.is_full()
    }

    pub fn handle_bb_subcmd_60(&mut self, handler: &mut BlockHandler, m: BbSubCmd60) -> Result<(), PartyError> {
        // We'll eventually do more on this.
        let cid = handler.client_id;
        match m {
            BbSubCmd60::Bb60ReqExp { data: r, .. } => {
                self.handle_bb_60_req_exp(handler, r);
            },
            _ => ()
        }
        self.bb_broadcast(handler, Some(cid), m.into())
    }

    pub fn handle_bb_subcmd_62(&mut self, _handler: &mut BlockHandler, _m: BbSubCmd62) -> Result<(), PartyError> {
        // This we do NOT propagate.
        Ok(())
    }

    pub fn handle_bb_subcmd_6c(&mut self, handler: &mut BlockHandler, m: BbSubCmd6C) -> Result<(), PartyError> {
        let cid = handler.client_id;
        self.bb_broadcast(handler, Some(cid), m.into())
    }

    pub fn handle_bb_subcmd_6d(&mut self, _handler: &mut BlockHandler, _m: BbSubCmd6D) -> Result<(), PartyError> {
        // This we do NOT propagate.
        Ok(())
    }

    pub fn handle_bb_60_req_exp(&mut self, handler: &mut BlockHandler, m: Bb60ReqExp) {
        let cid = handler.client_id;
        debug!("Client {} request exp: {:?}", cid, m);
        if let Some(ref enemy) = self.enemies.get(m.enemy_id as usize) {
            // TODO handle levelups with player level table
            let bp = {
                match self.episode {
                    1 => handler.battle_params.get_ep1(enemy.param_entry, self.single_player).cloned(),
                    2 => handler.battle_params.get_ep2(enemy.param_entry, self.single_player).cloned(),
                    3 => handler.battle_params.get_ep4(enemy.param_entry, self.single_player).cloned(),
                    _ => unreachable!()
                }
            };

            if let Some(bp) = bp {
                info!("Client {} request verified; +{} EXP for killing {} ({})", cid, bp.exp, enemy.name, m.enemy_id);
                self.award_exp(cid, handler, bp.exp);
            } else {
                error!("Battle param entry for enemy id {} doesn't exist", m.enemy_id);
                return
            }
        } else {
            warn!("Client {} tried to gain exp for an enemy that doesn't exist: {}", cid, m.enemy_id);
            return
        }
        // handler.send_to_client(cid, Message::BbSubCmd60(0, BbSubCmd60::Bb60GiveExp { client_id: 0, unused: 0, data: Bb60GiveExp(25) }));
        // send a level up as a test
        // let mut lup: Bb60LevelUp = Bb60LevelUp::default();
        // {
        //     let cr = handler.get_client_state(cid).unwrap();
        //     let ref mut c = cr.borrow_mut();
        //     if let Some(ref mut ch) = c.full_char {
        //         ch.chara.level += 1;
        //         lup.level = ch.chara.level;
        //     }
        // }
        // handler.send_to_client(cid, Message::BbSubCmd60(0, BbSubCmd60::Bb60LevelUp { client_id: 0, unused: 0, data: lup }));
    }

    fn award_exp(&self, client: usize, handler: &mut BlockHandler, exp: u32) {
        // TODO check for levelup using level table
        {
            let cr = handler.get_client_state(client).unwrap();
            let ref mut client = cr.borrow_mut();
            client.full_char.as_mut().unwrap().chara.exp += exp;
        }
        handler.send_to_client(client, Message::BbSubCmd60(0, BbSubCmd60::Bb60GiveExp { client_id: 0, unused: 0, data: Bb60GiveExp(exp) }))
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

    fn random_variants_ep1(maps: &Ep1Areas, event: u16) -> (Vec<u32>, Vec<InstanceEnemy>) {
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

        variants[2] = (random::<usize>() % maps.wilds1.len()) as u32;
        variants[5] = (random::<usize>() % maps.wilds2.len()) as u32;
        variants[7] = (random::<usize>() % maps.wilds3.len()) as u32;
        variants[9] = (random::<usize>() % maps.wilds4.len()) as u32;
        variants[11] = (random::<usize>() % maps.crater.len()) as u32;
        variants[12] = (random::<usize>() % maps.desert1.len()) as u32;
        variants[15] = (random::<usize>() % maps.desert2.len()) as u32;
        variants[16] = (random::<usize>() % maps.desert3.len()) as u32;

        Party::append_enemies(&maps.wilds1[variants[2] as usize].enemies, &mut enemies, 3, event, false);
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

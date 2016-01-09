//! Where the majority of the game occurs.

pub mod error;

use psomsg::bb::Message as BbMsg;
use psomsg::bb::*;

use super::handler::BlockHandler;

use self::error::PartyError;

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
    leader_id: u8
}

impl Party {
    pub fn new(name: &str, password: Option<&str>, episode: u8, difficulty: u8, battle: bool, challenge: bool, single_player: bool) -> Party {
        Party {
            name: name.to_owned(),
            password: password.map(|s| s.to_owned()),
            episode: episode,
            difficulty: difficulty,
            battle: battle,
            challenge: challenge,
            single_player: single_player,
            members: Default::default(),
            leader_id: 0
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
        // let cr = self.get_client_state(self.client_id).unwrap();
        // let c = cr.borrow();
        //
        // let mut l = BbGameJoin::default();
        // l.client_id = 0;
        // l.leader_id = 0;
        // l.one = 1;
        // l.one2 = 1;
        // l.difficulty = m.difficulty;
        // l.episode = m.episode;
        // let mut ph = PlayerHdr::default();
        // ph.tag = 0x00010000;
        // ph.guildcard = c.bb_guildcard;
        // ph.client_id = 0;
        // ph.name = c.full_char.as_ref().unwrap().name.clone();
        // l.players.push(ph);
        // let r: Message = Message::BbGameJoin(0, l);
        // self.sender.send((self.client_id, r).into()).unwrap();

        info!("Adding client {} to party {}", player, self.name);

        let cr = handler.get_client_state(player).unwrap();
        let c = cr.borrow();

        let mut l = BbGameJoin::default();
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

    pub fn handle_bb_60_req_exp(&mut self, handler: &mut BlockHandler, _m: Bb60ReqExp) {
        // TODO actual exp from BattleParamEntry
        let cid = handler.client_id;
        handler.send_to_client(cid, Message::BbSubCmd60(0, BbSubCmd60::Bb60GiveExp { client_id: 0, unused: 0, data: Bb60GiveExp(25) }));
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
}

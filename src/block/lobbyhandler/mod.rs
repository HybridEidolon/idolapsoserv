//! Lobby handler. Lobbies can handle a max of 12 players (or the client
//! crashes).

use super::handler::BlockHandler;

use psomsg::bb::Message as BbMsg;
use psomsg::bb::*;

pub mod error;
pub mod event;

use self::error::LobbyError;

const MAX_PLAYERS: usize = 12;

#[derive(Clone, Debug)]
pub struct Lobby {
    player_count: usize,
    players: [Option<usize>; MAX_PLAYERS],
    lobby_num: u8,
    block_num: u16,
    event: u16,
    leader_id: u8
}

impl Lobby {

    /// `num` is the lobby number sent to joiners, `event` is the seasonal
    /// event for this lobby (yes, lobbies can have different events on the
    /// same block). `num` is 0-14. (+1 for in-client number)
    pub fn new(num: u8, block: u16, event: u16) -> Lobby {
        // TODO event as type-safe enum to prevent client crashes
        Lobby {
            player_count: 0,
            players: [None; 12],
            lobby_num: num,
            block_num: block,
            event: event,
            leader_id: 0
        }
    }

    /// Broadcasts a Blue Burst message to all players in the lobby. Performs
    /// conversion on the message to the appropriate client versions if
    /// necessary. If sent_by is `Some`, that client will not receive the
    /// message.
    pub fn bb_broadcast(&self, handler: &mut BlockHandler, sent_by: Option<usize>, msg: BbMsg) -> Result<(), LobbyError> {
        for co in self.players.iter() {
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

    /// Adds a player to this lobby.
    pub fn add_player(&mut self, handler: &mut BlockHandler, player: usize) -> Result<(), LobbyError> {
        if self.has_player(player) {
            return Err(LobbyError::AlreadyInLobby)
        }

        info!("Adding client {} to lobby {}:{}", player, self.block_num, self.lobby_num + 1);

        let new_client_id: u8 = match self.find_first_empty() {
            Some(slot) => {
                if self.is_empty() {
                    // This player is going to become the lobby leader.
                    self.leader_id = slot;
                }
                slot
            },
            None => {
                return Err(LobbyError::IsFull)
            }
        };

        self.players[new_client_id as usize] = Some(player);

        // Tell other clients that this player is joining.
        {
            let mut lm: LobbyMember = LobbyMember::default();
            let cr = handler.get_client_state(player).unwrap();
            let ref c = cr.borrow();
            lm.hdr.tag = 0x00010000;
            lm.hdr.guildcard = c.bb_guildcard;
            lm.hdr.client_id = new_client_id as u32;
            lm.hdr.name = c.full_char.as_ref().unwrap().chara.name.clone();
            lm.inventory = c.full_char.as_ref().unwrap().inv.clone();
            lm.data = c.full_char.as_ref().unwrap().chara.clone();

            for (slot, co) in self.players.iter().enumerate() {
                match co {
                    &Some(c) => {
                        if c != player {
                            let mut lam = LobbyAddMember::default();
                            lam.client_id = slot as u8; // yeah, confusing, but AddMember can add multiple lobby members at once
                            lam.leader_id = self.leader_id as u8;
                            lam.one = 0;
                            lam.lobby_num = self.lobby_num;
                            lam.block_num = self.block_num;
                            lam.event = self.event;
                            lam.members.push(lm.clone());

                            handler.send_to_client(c, Message::LobbyAddMember(1, lam));
                        }
                    },
                    _ => ()
                }
            }
        }

        // Tell this player that they are joining the lobby.
        {
            let mut members = Vec::new();
            for (slot, co) in self.players.iter().enumerate() {
                match co {
                    &Some(cid) => {
                        let mut lm: LobbyMember = LobbyMember::default();
                        let cr = handler.get_client_state(cid).unwrap();
                        let ref c = cr.borrow();
                        let fc = c.full_char.as_ref().unwrap();
                        lm.hdr.tag = 0x00010000;
                        lm.hdr.guildcard = c.bb_guildcard;
                        lm.hdr.client_id = slot as u32;
                        lm.hdr.name = fc.chara.name.clone();
                        lm.inventory = fc.inv.clone();
                        lm.data = fc.chara.clone();
                        members.push(lm);
                    }
                    _ => ()
                }
            }
            let mut lj = LobbyJoin::default();
            lj.client_id = new_client_id;
            lj.leader_id = self.leader_id;
            lj.one = 1;
            lj.lobby_num = self.lobby_num;
            lj.block_num = self.block_num;
            lj.event = self.event;
            lj.members = members;
            handler.send_to_client(player, Message::LobbyJoin(lj.members.len() as u32, lj));

            // TODO send the arrow list too...

            let cr = handler.get_client_state(player).unwrap();
            let c = cr.borrow();
            let fc = c.full_char.as_ref().unwrap();
            // send this player's quest data, don't know why
            let r = Message::BbSubCmd60(0, BbSubCmd60::QuestData1 {
                client_id: 0,
                unused: 0,
                data: QuestData1(fc.quest_data1.clone())
            });
            handler.send_to_client(player, r);
        }

        Ok(())
    }

    /// Removes a player from this lobby. All other lobby members will be told
    /// about the player leaving, but this player will not receive anything.
    /// You must remember to tell the player where they are going!
    pub fn remove_player(&mut self, handler: &mut BlockHandler, player: usize) -> Result<(), LobbyError> {
        if !self.has_player(player) {
            return Err(LobbyError::NotInLobby)
        }

        info!("Removing client {} from lobby {}:{}", player, self.block_num, self.lobby_num + 1);

        if self.num_players() == 1 {
            // lobby is empty now...
            self.leader_id = 0;
            for p in self.players.iter_mut() {
                *p = None;
            }
            Ok(())
        } else {
            let player_client_id = self.client_id_for_player(player).unwrap();

            if self.leader_id == player_client_id {
                // need to elect a new leader
                self.leader_id = self.find_first_player_not_matching(player).unwrap().0;
            }

            self.players[player_client_id as usize] = None;

            self.bb_broadcast(handler, Some(player), BbMsg::LobbyLeave(0, LobbyLeave {
                client_id: player_client_id,
                leader_id: self.leader_id,
                padding: 0
            }))
        }
    }

    /// Get the current player count.
    pub fn num_players(&self) -> usize {
        let mut count = 0;
        for i in 0..MAX_PLAYERS {
            match self.players[i] {
                Some(_) => {
                    count += 1;
                },
                _ => ()
            }
        }

        count
    }

    /// If this lobby is currently full.
    pub fn is_full(&self) -> bool {
        self.num_players() >= 12
    }

    /// If this lobby is currently empty.
    pub fn is_empty(&self) -> bool {
        self.num_players() == 0
    }

    /// Sets the event without reloading all clients. Will only take effect for
    /// new clients.
    pub fn set_event(&mut self, event: u16) {
        self.event = event;
    }

    /// Whether or not this lobby has this player.
    pub fn has_player(&self, client: usize) -> bool {
        for i in 0..MAX_PLAYERS {
            match self.players[i] {
                Some(cid) => {
                    if cid == client {
                        return true
                    }
                },
                None => continue
            }
        }
        false
    }

    pub fn lobby_num(&self) -> u8 { self.lobby_num }
    pub fn block_num(&self) -> u16 { self.block_num }
    pub fn event_num(&self) -> u16 { self.event }

    /// Sets the event and reloads all clients.
    pub fn set_event_reload(&mut self, handler: &mut BlockHandler, event: u16) -> Result<(), LobbyError> {
        self.event = event;
        error!("Reloading after setting event is not implemented! We won't panic over it though.\n at {}:{}", file!(), line!());
        let cid = handler.client_id;
        handler.send_error(cid, "\tEPlease re-enter the\nlobby to see the\nnew event.");
        Ok(())
    }

    pub fn handle_bb_subcmd_60(&mut self, handler: &mut BlockHandler, m: BbSubCmd60) -> Result<(), LobbyError> {
        // We'll eventually do more on this.
        let cid = handler.client_id;
        self.bb_broadcast(handler, Some(cid), m.into())
    }

    pub fn handle_bb_subcmd_62(&mut self, _handler: &mut BlockHandler, _dest: u32, _m: BbSubCmd62) -> Result<(), LobbyError> {
        // This we do NOT propagate.
        Ok(())
    }

    pub fn handle_bb_subcmd_6c(&mut self, handler: &mut BlockHandler, m: BbSubCmd6C) -> Result<(), LobbyError> {
        let cid = handler.client_id;
        self.bb_broadcast(handler, Some(cid), m.into())
    }

    pub fn handle_bb_subcmd_6d(&mut self, _handler: &mut BlockHandler, _dest: u32, _m: BbSubCmd6D) -> Result<(), LobbyError> {
        // This we do NOT propagate.
        Ok(())
    }

    /// `None` if this lobby is full. `Some(client ID slot)` if not.
    fn find_first_empty(&self) -> Option<u8> {
        for (i, po) in self.players.iter().enumerate() {
            match po {
                &None => {
                    return Some(i as u8)
                },
                _ => continue
            }
        }
        None
    }

    /// `None` if there are no players. (Lobby client ID, connection ID)
    // fn find_first_player(&self) -> Option<(u8, usize)> {
    //     for (i, po) in self.players.iter().enumerate() {
    //         match po {
    //             &Some(cid) => {
    //                 return Some((i as u8, cid))
    //             },
    //             _ => continue
    //         }
    //     }
    //     None
    // }

    fn find_first_player_not_matching(&self, player: usize) -> Option<(u8, usize)> {
        for (i, po) in self.players.iter().enumerate() {
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
        for (i, po) in self.players.iter().enumerate() {
            match po {
                &Some(cid) if cid == player => {
                    return Some(i as u8)
                },
                _ => continue
            }
        }
        None
    }
}

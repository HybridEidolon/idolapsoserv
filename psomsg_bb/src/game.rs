use psoserial::Serial;

use std::io::{Read, Write};
use std::io;

use psomsg_common::util::*;
use super::player::*;
use super::lobby::*;

#[derive(Clone, Debug, Default)]
pub struct BbCreateGame {
    pub name: String,
    pub password: String,
    pub difficulty: u8,
    pub battle: u8,
    pub challenge: u8,
    pub episode: u8,
    pub single_player: u8
}
impl Serial for BbCreateGame {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(u64::deserialize(src)); //padding
        let name = try!(read_utf16_len(16*2, src));
        let password = try!(read_utf16_len(16*2, src));
        let difficulty = try!(Serial::deserialize(src));
        let battle = try!(Serial::deserialize(src));
        let challenge = try!(Serial::deserialize(src));
        let episode = try!(Serial::deserialize(src));
        let single_player = try!(Serial::deserialize(src));
        try!(src.read(&mut [0; 3])); //padding
        Ok(BbCreateGame {
            name: name,
            password: password,
            difficulty: difficulty,
            battle: battle,
            challenge: challenge,
            episode: episode,
            single_player: single_player
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbGameJoin {
    pub maps: Vec<u32>,
    pub players: Vec<PlayerHdr>,
    pub client_id: u8,
    pub leader_id: u8,
    pub one: u8,
    pub difficulty: u8,
    pub battle: u8,
    pub event: u8,
    pub section: u8,
    pub challenge: u8,
    pub rand_seed: u32,
    pub episode: u8,
    pub one2: u8,
    pub single_player: u8,
    pub unused: u8
}
impl Serial for BbGameJoin {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.maps, 0x20, dst));
        try!(write_array(&self.players, 4, dst));
        try!(self.client_id.serialize(dst));
        try!(self.leader_id.serialize(dst));
        try!(self.one.serialize(dst));
        try!(self.difficulty.serialize(dst));
        try!(self.battle.serialize(dst));
        try!(self.event.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.challenge.serialize(dst));
        try!(self.rand_seed.serialize(dst));
        try!(self.episode.serialize(dst));
        try!(self.one2.serialize(dst));
        try!(self.single_player.serialize(dst));
        try!(self.unused.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbGameList {
    pub games: Vec<BbGameListEntry>
}

impl Serial for BbGameList {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        for g in self.games.iter() {
            try!(g.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let mut games = Vec::new();
        loop {
            match BbGameListEntry::deserialize(src) {
                Ok(g) => games.push(g),
                Err(_) => break
            }
        }
        Ok(BbGameList { games: games })
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbGameListEntry {
    // uint32_t menu_id;
    // uint32_t item_id;
    // uint8_t difficulty;
    // uint8_t players;
    // uint16_t name[16];
    // uint8_t episode;
    // uint8_t flags;
    pub menu_id: u32,
    pub item_id: u32,
    pub difficulty: u8,
    pub players: u8,
    pub name: String,
    pub episode: u8,
    pub flags: u8
}
impl Serial for BbGameListEntry {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.menu_id.serialize(dst));
        try!(self.item_id.serialize(dst));
        try!(self.difficulty.serialize(dst));
        try!(self.players.serialize(dst));
        try!(write_utf16_len(&self.name, 32, dst));
        try!(self.episode.serialize(dst));
        try!(self.flags.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let menu_id = try!(Serial::deserialize(src));
        let item_id = try!(Serial::deserialize(src));
        let difficulty = try!(Serial::deserialize(src));
        let players = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(32, src));
        let episode = try!(Serial::deserialize(src));
        let flags = try!(Serial::deserialize(src));
        Ok(BbGameListEntry {
            menu_id: menu_id,
            item_id: item_id,
            difficulty: difficulty,
            players: players,
            name: name,
            episode: episode,
            flags: flags
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbGameName(pub String);
impl Serial for BbGameName {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbGameName(try!(read_utf16(src))))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbPlayerLeaveGame(pub BbPlayerData);
impl Serial for BbPlayerLeaveGame {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbPlayerLeaveGame(try!(Serial::deserialize(src))))
    }
}

derive_serial! {
    BbGameLeave {
        pub client_id: u8,
        pub leader_id: u8,
        pub padding: u16
    }
}

#[derive(Clone, Debug)]
pub struct BbGameAddMember {
    pub client_id: u8,
    pub leader_id: u8,
    pub one: u8,
    pub lobby_num: u8,
    pub block_num: u16,
    pub event: u16,
    pub member: LobbyMember
}
impl Serial for BbGameAddMember {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.client_id.serialize(dst));
        try!(self.leader_id.serialize(dst));
        try!(self.one.serialize(dst));
        try!(self.lobby_num.serialize(dst));
        try!(self.block_num.serialize(dst));
        try!(self.event.serialize(dst));
        try!(0u32.serialize(dst)); //padding
        try!(self.member.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}
impl Default for BbGameAddMember {
    fn default() -> Self {
        BbGameAddMember {
            client_id: Default::default(),
            leader_id: Default::default(),
            one: 1,
            lobby_num: 0xFF,
            block_num: 1,
            event: 1,
            member: Default::default()
        }
    }
}

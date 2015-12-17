use std::io;
use std::io::{Read, Write};

//use ::util::*;
use ::Serial;
use super::chara::*;
use super::player::*;

#[derive(Clone, Debug)]
pub struct LobbyJoin {
    pub client_id: u8,
    pub leader_id: u8,
    pub lobby_num: u8,
    pub block_num: u16,
    pub event: u16,
    pub members: Vec<LobbyMember>
}
impl Serial for LobbyJoin {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.client_id.serialize(dst));
        try!(self.leader_id.serialize(dst));
        try!(1u8.serialize(dst));
        try!(self.lobby_num.serialize(dst));
        try!(self.block_num.serialize(dst));
        try!(self.event.serialize(dst));
        for i in self.members.iter() {
            try!(i.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}
impl Default for LobbyJoin {
    fn default() -> Self {
        LobbyJoin {
            client_id: Default::default(),
            leader_id: Default::default(),
            lobby_num: Default::default(),
            block_num: Default::default(),
            event: Default::default(),
            members: Vec::new()
        }
    }
}

pub type LobbyAddMember = LobbyJoin;

derive_serial! {
    LobbyMember {
        pub hdr: PlayerHdr,
        pub inventory: Inventory,
        pub data: BbChar
    }
}
impl Default for LobbyMember {
    fn default() -> Self {
        LobbyMember {
            hdr: Default::default(),
            inventory: Default::default(),
            data: Default::default()
        }
    }
}

use std::io;
use std::io::{Read, Write};

use psoserial::Serial;
use super::chara::*;
use super::player::*;

#[derive(Clone, Debug)]
pub struct LobbyJoin {
    pub client_id: u8,
    pub leader_id: u8,
    pub one: u8,
    pub lobby_num: u8,
    pub block_num: u16,
    pub event: u16,
    pub members: Vec<LobbyMember>
}
impl Serial for LobbyJoin {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.client_id.serialize(dst));
        try!(self.leader_id.serialize(dst));
        try!(self.one.serialize(dst));
        try!(self.lobby_num.serialize(dst));
        try!(self.block_num.serialize(dst));
        try!(self.event.serialize(dst));
        try!(0u32.serialize(dst)); //padding
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
            one: 1,
            lobby_num: Default::default(),
            block_num: Default::default(),
            event: Default::default(),
            members: Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct LobbyAddMember {
    pub client_id: u8,
    pub leader_id: u8,
    pub one: u8,
    pub lobby_num: u8,
    pub block_num: u16,
    pub event: u16,
    pub members: Vec<LobbyMember>
}
impl Serial for LobbyAddMember {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.client_id.serialize(dst));
        try!(self.leader_id.serialize(dst));
        try!(self.one.serialize(dst));
        try!(self.lobby_num.serialize(dst));
        try!(self.block_num.serialize(dst));
        try!(self.event.serialize(dst));
        try!(0u32.serialize(dst)); //padding
        for i in self.members.iter() {
            try!(i.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}
impl Default for LobbyAddMember {
    fn default() -> Self {
        LobbyAddMember {
            client_id: Default::default(),
            leader_id: Default::default(),
            one: 1,
            lobby_num: Default::default(),
            block_num: Default::default(),
            event: Default::default(),
            members: Vec::new()
        }
    }
}

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

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use psoserial::Serial;
    #[test]
    fn test_lobby_join_size() {
        let l = LobbyJoin::default();
        let mut c = Cursor::new(Vec::new());
        l.serialize(&mut c).unwrap();
        assert_eq!(c.position(), 0x14 - 8);

        let mut l = LobbyJoin::default();
        l.members.push(LobbyMember::default());
        let mut c = Cursor::new(Vec::new());
        l.serialize(&mut c).unwrap();
        assert_eq!(c.position(), 1324);
    }
}

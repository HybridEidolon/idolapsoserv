use ::Serial;

use std::io;
use std::io::{Read, Write};
use std::net::Ipv4Addr;

use byteorder::{LittleEndian as LE, WriteBytesExt};

use ::util::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Goodbye;
impl_unit_serial!(Goodbye);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Redirect {
    // uint32_t ip_addr;       /* Big-endian */
    // uint16_t port;          /* Little-endian */
    // uint8_t padding[2];
    pub ip: Ipv4Addr,
    pub port: u16
}

impl Serial for Redirect {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.ip.serialize(dst));
        try!(dst.write_u16::<LE>(self.port));
        try!(dst.write_u16::<LE>(0)); // padding
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LargeMsg(pub String);
impl Serial for LargeMsg {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Timestamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub msec: u16
}
impl Serial for Timestamp {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_ascii_len(&format!("{:04}:{:02}:{:02}: {:02}:{:02}:{:02}.{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.msec),
            28, dst));
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct ShipList(pub Vec<ShipListItem>);
impl Serial for ShipList {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        for i in self.0.iter() {
            try!(i.serialize(dst))
        }
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

pub type BlockList = ShipList;

#[derive(Clone, Debug)]
pub struct LobbyList {
    pub items: Vec<(u32, u32)>
}
impl Serial for LobbyList {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        for i in self.items.iter() {
            let &(ref menu_id, ref item_id) = i;
            try!(menu_id.serialize(dst));
            try!(item_id.serialize(dst));
            try!(0u32.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct LobbyArrowList(pub Vec<(u32, u32, u32)>);
impl Serial for LobbyArrowList {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        for i in self.0.iter() {
            let &(ref tag, ref guildcard, ref arrow) = i;
            try!(tag.serialize(dst));
            try!(guildcard.serialize(dst));
            try!(arrow.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

derive_serial!(CharDataRequest);

#[derive(Clone, Copy, Debug)]
pub struct MenuSelect(pub u32, pub u32);
impl Serial for MenuSelect {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        try!(self.1.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(MenuSelect(try!(u32::deserialize(src)), try!(u32::deserialize(src))))
    }
}

#[derive(Clone, Debug)]
pub struct ShipListItem {
    pub menu_id: u32,
    pub item_id: u32,
    pub flags: u16,
    pub name: String //0x22 bytes, 0x11 chars
}
impl Serial for ShipListItem {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.menu_id.serialize(dst));
        try!(self.item_id.serialize(dst));
        try!(self.flags.serialize(dst));
        try!(write_utf16_len(&self.name, 0x22, dst));
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

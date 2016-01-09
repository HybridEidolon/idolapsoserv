//! Message structures that are commonly formed between all versions of PSO.,
//! and common utilities.

#[macro_use] extern crate psoserial;
extern crate byteorder;
#[macro_use] extern crate log;
extern crate encoding;

use psoserial::Serial;

use std::io;
use std::io::{Read, Write};
use std::net::Ipv4Addr;

use byteorder::{LittleEndian as LE, BigEndian as BE, WriteBytesExt, ReadBytesExt};

pub mod util;

use ::util::*;

derive_serial!(Goodbye);

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

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let ip: Ipv4Addr = try!(src.read_u32::<BE>()).into();
        let port = try!(Serial::deserialize(src));
        try!(u16::deserialize(src));
        Ok(Redirect {
            ip: ip,
            port: port
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LargeMsg(pub String);
impl Serial for LargeMsg {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let msg = try!(read_utf16(src));
        Ok(LargeMsg(msg))
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
        let timestamp_string = format!("{:04}:{:02}:{:02}: {:02}:{:02}:{:02}.{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.msec);
        try!(write_ascii_len(&timestamp_string, 28, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let timestamp_string = try!(read_ascii_len(28, src));
        if timestamp_string.len() != 24 {
            return Err(io::Error::new(io::ErrorKind::Other, "Timestamp string is not long enough"))
        }
        let year: u16 = try!(timestamp_string[0..4].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let month: u8 = try!(timestamp_string[5..7].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let day: u8 = try!(timestamp_string[8..10].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let hour: u8 = try!(timestamp_string[12..14].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let minute: u8 = try!(timestamp_string[15..17].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let second: u8 = try!(timestamp_string[18..20].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let msec: u16 = try!(timestamp_string[21..24].parse().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        Ok(Timestamp {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            msec: msec
        })
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

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let mut ships = Vec::new();
        loop {
            match ShipListItem::deserialize(src) {
                Ok(s) => ships.push(s),
                _ => break
            }
        }
        Ok(ShipList(ships))
    }
}

#[derive(Clone, Debug)]
pub struct BlockList(pub Vec<ShipListItem>);
impl Serial for BlockList {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        for i in self.0.iter() {
            try!(i.serialize(dst))
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let mut blocks = Vec::new();
        loop {
            match ShipListItem::deserialize(src) {
                Ok(s) => blocks.push(s),
                _ => break
            }
        }
        Ok(BlockList(blocks))
    }
}

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

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let mut lobbies = Vec::new();
        loop {
            let v1 = match u32::deserialize(src) {
                Ok(v) => v,
                Err(_) => break
            };
            let v2 = match u32::deserialize(src) {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            match u32::deserialize(src) {
                Ok(_) => (),
                Err(e) => return Err(e)
            }
            lobbies.push((v1, v2));
        }
        Ok(LobbyList {
            items: lobbies
        })
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

#[derive(Clone, Copy, Debug)]
pub struct LobbyChange(pub u32, pub u32);
impl Serial for LobbyChange {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        try!(self.1.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(LobbyChange(try!(u32::deserialize(src)), try!(u32::deserialize(src))))
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

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let menu_id = try!(Serial::deserialize(src));
        let item_id = try!(Serial::deserialize(src));
        let flags = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(0x22, src));
        Ok(ShipListItem {
            menu_id: menu_id,
            item_id: item_id,
            flags: flags,
            name: name
        })
    }
}

derive_serial!(Ping);

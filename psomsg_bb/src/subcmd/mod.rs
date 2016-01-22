//! Subcommand message types, for message 0x60

use std::io;
use std::io::{Read, Write};

use psoserial::Serial;
use psomsg_common::util::*;

pub mod wrapper;
pub mod sub62;
pub mod sub6d;

pub use self::wrapper::{BbSubCmd60, BbSubCmd62, BbSubCmd6C, BbSubCmd6D};
pub use self::sub62::*;
pub use self::sub6d::*;

#[derive(Clone, Debug)]
pub struct QuestData1(pub Vec<u8>);
impl Serial for QuestData1 {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.0, 0x0208, dst));
        try!(0u32.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(read_array(0x0208, src));
        try!(u32::deserialize(src));
        Ok(QuestData1(data))
    }
}
impl Default for QuestData1 {
    fn default() -> QuestData1 {
        QuestData1(vec![0u8; 0x0208])
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bb60GiveExp(pub u32);
impl Serial for Bb60GiveExp {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(Bb60GiveExp(try!(Serial::deserialize(src))))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bb60ReqExp {
    pub enemy_id: u16,
    pub client_id: u8,
    pub unused1: u8,
    pub last_hitter: u8,
    pub unused2: u8,
    pub unused3: u16
}
impl Serial for Bb60ReqExp {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.enemy_id.serialize(dst));
        try!(self.client_id.serialize(dst));
        try!(self.unused1.serialize(dst));
        try!(self.last_hitter.serialize(dst));
        try!(self.unused2.serialize(dst));
        try!(self.unused3.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(Bb60ReqExp {
            enemy_id: try!(Serial::deserialize(src)),
            client_id: try!(Serial::deserialize(src)),
            unused1: try!(Serial::deserialize(src)),
            last_hitter: try!(Serial::deserialize(src)),
            unused2: try!(Serial::deserialize(src)),
            unused3: try!(Serial::deserialize(src)),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bb60LevelUp {
    pub atp: u16,
    pub mst: u16,
    pub evp: u16,
    pub hp: u16,
    pub dfp: u16,
    pub ata: u16,
    pub level: u32
}
impl Serial for Bb60LevelUp {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.atp.serialize(dst));
        try!(self.mst.serialize(dst));
        try!(self.evp.serialize(dst));
        try!(self.hp.serialize(dst));
        try!(self.dfp.serialize(dst));
        try!(self.ata.serialize(dst));
        try!(self.level.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let atp = try!(Serial::deserialize(src));
        let mst = try!(Serial::deserialize(src));
        let evp = try!(Serial::deserialize(src));
        let hp = try!(Serial::deserialize(src));
        let dfp = try!(Serial::deserialize(src));
        let ata = try!(Serial::deserialize(src));
        let level = try!(Serial::deserialize(src));
        Ok(Bb60LevelUp {
            atp: atp,
            mst: mst,
            evp: evp,
            hp: hp,
            dfp: dfp,
            ata: ata,
            level: level
        })
    }
}

#[derive(Clone, Debug)]
pub struct Bb60DoneBurst {
    pub data: Vec<u8>
}
impl Serial for Bb60DoneBurst {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.data, 12, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(read_array(12, src));
        Ok(Bb60DoneBurst {
            data: data
        })
    }
}
impl Default for Bb60DoneBurst {
    fn default() -> Bb60DoneBurst {
        Bb60DoneBurst {
            data: vec![0; 12]
        }
    }
}

derive_serial_default! {
    Bb60DropItem {
        // uint16_t unk;
        // uint16_t area;
        // uint32_t item_id;
        // float x;
        // float y;
        // float z;
        pub unk: u16,
        pub area: u16,
        pub item_id: u32,
        pub x: f32,
        pub y: f32,
        pub z: f32
    }
}

derive_serial_default! {
    Bb60DeleteItem {
        pub item_id: u32,
        pub amount: u32
    }
}

derive_serial_default! {
    Bb60DestroyItem {
        pub item_id: u32,
        pub amount: u32
    }
}

derive_serial_default! {
    Bb60DropPos {
        pub area: u32,
        pub x: f32,
        pub z: f32,
        pub item_id: u32,
        pub amount: u32
    }
}

derive_serial_default! {
    Bb60DropStack {
        pub area: u32,
        pub x: f32,
        pub z: f32,
        pub item: [u8; 12],
        pub item_id: u32,
        pub item2: [u8; 4]
    }
}

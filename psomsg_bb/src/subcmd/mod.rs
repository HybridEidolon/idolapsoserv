//! Subcommand message types, for message 0x60

use std::io;
use std::io::{Read, Write};

use psoserial::Serial;
use psomsg_common::util::*;

pub mod wrapper;
pub mod sub62;

pub use self::wrapper::{BbSubCmd60, BbSubCmd62, BbSubCmd6C, BbSubCmd6D};
pub use self::sub62::*;

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

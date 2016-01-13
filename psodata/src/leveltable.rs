//! Stats per level for each class.

use psoserial::Serial;

use std::io::{Read, Write};
use std::io;

use psoserial::util::*;

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub struct LevelEntry {
    pub atp: u8,
    pub mst: u8,
    pub evp: u8,
    pub hp: u8,
    pub dfp: u8,
    pub ata: u8,
    pub unk: u16,
    pub exp: u32
}
impl Serial for LevelEntry {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.atp.serialize(dst));
        try!(self.mst.serialize(dst));
        try!(self.evp.serialize(dst));
        try!(self.hp.serialize(dst));
        try!(self.dfp.serialize(dst));
        try!(self.ata.serialize(dst));
        try!(self.unk.serialize(dst));
        try!(self.exp.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let atp = try!(Serial::deserialize(src));
        let mst = try!(Serial::deserialize(src));
        let evp = try!(Serial::deserialize(src));
        let hp = try!(Serial::deserialize(src));
        let dfp = try!(Serial::deserialize(src));
        let ata = try!(Serial::deserialize(src));
        let unk = try!(Serial::deserialize(src));
        let exp = try!(Serial::deserialize(src));
        Ok(LevelEntry {
            atp: atp,
            mst: mst,
            evp: evp,
            hp: hp,
            dfp: dfp,
            ata: ata,
            unk: unk,
            exp: exp
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct LevelTable {
    pub start_stats: Vec<LevelEntry>,
    unk: Vec<u8>,
    pub levels: Vec<Vec<LevelEntry>>
}
impl Serial for LevelTable {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.start_stats, 12, dst));
        try!(write_array(&self.unk, 48, dst));
        let padding = 12 - self.levels.len();
        for l in self.levels.iter() {
            try!(write_array(&l, 200, dst));
        }
        for _ in 0..padding {
            let d = LevelEntry::default();
            for _ in 0..200 {
                try!(d.serialize(dst));
            }
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let start_stats = try!(read_array(12, src));
        let unk = try!(read_array(48, src));
        let mut levels = Vec::new();
        for _ in 0..12 {
            let mut curclass = Vec::new();
            for _ in 0..200 {
                let level = try!(Serial::deserialize(src));
                curclass.push(level);
            }
            levels.push(curclass);
        }

        Ok(LevelTable {
            start_stats: start_stats,
            unk: unk,
            levels: levels
        })
    }
}

//! BattleParamEntry structure. A BattleParamEntry file has room for 0x180 entries.

use super::Parse;

use std::io::{Read, Write};
use std::io;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BattleParam {
    pub atp: u16,
    pub int: u16,
    pub evp: u16,
    pub hp: u16,
    pub dfp: u16,
    pub ata: u16,
    pub lck: u16,
    pub unk: [u8; 14],
    pub exp: u32,
    pub difficulty: u32
}

impl Parse for BattleParam {
    fn read(src: &mut Read) -> io::Result<BattleParam> {
        let atp = try!(src.read_u16::<LE>());
        let int = try!(src.read_u16::<LE>());
        let evp = try!(src.read_u16::<LE>());
        let hp  = try!(src.read_u16::<LE>());
        let dfp = try!(src.read_u16::<LE>());
        let ata = try!(src.read_u16::<LE>());
        let lck = try!(src.read_u16::<LE>());
        let mut unk = [0; 14]; try!(src.read(&mut unk[..]));
        let exp = try!(src.read_u32::<LE>());
        let difficulty = try!(src.read_u32::<LE>());
        Ok(BattleParam {
            atp: atp,
            int: int,
            evp: evp,
            hp: hp,
            dfp: dfp,
            ata: ata,
            lck: lck,
            unk: unk,
            exp: exp,
            difficulty: difficulty
        })
    }

    fn write(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u16::<LE>(self.atp));
        try!(dst.write_u16::<LE>(self.int));
        try!(dst.write_u16::<LE>(self.evp));
        try!(dst.write_u16::<LE>(self.hp));
        try!(dst.write_u16::<LE>(self.dfp));
        try!(dst.write_u16::<LE>(self.ata));
        try!(dst.write_u16::<LE>(self.lck));
        try!(dst.write_all(&self.unk[..]));
        try!(dst.write_u32::<LE>(self.exp));
        try!(dst.write_u32::<LE>(self.difficulty));
        Ok(())
    }
}

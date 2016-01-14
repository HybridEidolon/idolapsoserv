//! BattleParamEntry structure. A BattleParamEntry file has room for 0x180 entries.

use psoserial::Serial;

use std::io::{Read, Write};
use std::io;
use std::fs::File;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Debug)]
pub struct BattleParamTables {
    ep1: Vec<BattleParam>,
    ep2: Vec<BattleParam>,
    ep4: Vec<BattleParam>,
    ep1_1p: Vec<BattleParam>,
    ep2_1p: Vec<BattleParam>,
    ep4_1p: Vec<BattleParam>
}

impl BattleParamTables {
    /// Load the complete Battle Param tables from a path. The files
    /// for every episode and offline mode must be available.
    pub fn load_from_files(path: &str) -> io::Result<BattleParamTables> {
        let ep1 = try!(load_params(try!(File::open(format!("{}/BattleParamEntry_on.dat", path)))));
        let ep2 = try!(load_params(try!(File::open(format!("{}/BattleParamEntry_lab_on.dat", path)))));
        let ep4 = try!(load_params(try!(File::open(format!("{}/BattleParamEntry_ep4_on.dat", path)))));
        let ep1_1p = try!(load_params(try!(File::open(format!("{}/BattleParamEntry.dat", path)))));
        let ep2_1p = try!(load_params(try!(File::open(format!("{}/BattleParamEntry_lab.dat", path)))));
        let ep4_1p = try!(load_params(try!(File::open(format!("{}/BattleParamEntry_ep4.dat", path)))));
        Ok(BattleParamTables {
            ep1: ep1,
            ep2: ep2,
            ep4: ep4,
            ep1_1p: ep1_1p,
            ep2_1p: ep2_1p,
            ep4_1p: ep4_1p
        })
    }

    pub fn get_ep1(&self, idx: usize, one_player: bool, difficulty: u8) -> Option<&BattleParam> {
        if one_player {
            self.ep1_1p.get(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep1.get(idx + 0x60 * (difficulty as usize))
        }
    }

    pub fn get_ep1_mut(&mut self, idx: usize, one_player: bool, difficulty: u8) -> Option<&mut BattleParam> {
        if one_player {
            self.ep1_1p.get_mut(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep1.get_mut(idx + 0x60 * (difficulty as usize))
        }
    }

    pub fn get_ep2(&self, idx: usize, one_player: bool, difficulty: u8) -> Option<&BattleParam> {
        if one_player {
            self.ep2_1p.get(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep2.get(idx + 0x60 * (difficulty as usize))
        }
    }

    pub fn get_ep2_mut(&mut self, idx: usize, one_player: bool, difficulty: u8) -> Option<&mut BattleParam> {
        if one_player {
            self.ep2_1p.get_mut(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep2.get_mut(idx + 0x60 * (difficulty as usize))
        }
    }

    pub fn get_ep4(&self, idx: usize, one_player: bool, difficulty: u8) -> Option<&BattleParam> {
        if one_player {
            self.ep4_1p.get(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep4.get(idx + 0x60 * (difficulty as usize))
        }
    }

    pub fn get_ep4_mut(&mut self, idx: usize, one_player: bool, difficulty: u8) -> Option<&mut BattleParam> {
        if one_player {
            self.ep4_1p.get_mut(idx + 0x60 * (difficulty as usize))
        } else {
            self.ep4.get_mut(idx + 0x60 * (difficulty as usize))
        }
    }
}

fn load_params<R: Read>(mut src: R) -> io::Result<Vec<BattleParam>> {
    let mut params: Vec<BattleParam> = Vec::new();
    for _ in 0..0x180 {
        let param = try!(BattleParam::deserialize(&mut src));
        params.push(param);
    }
    Ok(params)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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

impl Serial for BattleParam {
    fn deserialize(src: &mut Read) -> io::Result<BattleParam> {
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

    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
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

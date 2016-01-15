//! Item rare tables for GC/BB. The ItemRT.gsl is organized much in the same
//! way as the ItemPT archive.

use std::io::{Read, Write, Cursor};
use std::io;

use psoserial::Serial;
use psoserial::util::*;

/// Parsed rare table entry. There are 101 Enemy entries and 30 Box entries in
/// a full rare table file.
#[derive(Clone, Copy, Debug, Default)]
pub struct RtEntry {
    pub prob: u8,
    pub item_data: [u8; 3]
}
impl Serial for RtEntry {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.prob.serialize(dst));
        try!(self.item_data.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(RtEntry {
            prob: try!(Serial::deserialize(src)),
            item_data: try!(Serial::deserialize(src))
        })
    }
}

impl RtEntry {
    /// Derive the drop probability from the encoded value.
    ///
    /// This function was derived from the Sylverant codebase, which itself
    /// comes from research done by Lee, much like the rest of the data formats.
    pub fn probability(&self) -> f64 {
        let tmp = match (self.prob >> 3) as isize - 4 {
            v if v < 0 => 0,
            v => v
        };
        let expanded = ((2 << tmp) * ((self.prob & 7) + 7)) as f64;
        expanded / (0x100000000u64 as f64)
    }

    /// Derive item data from the encoded value.
    pub fn item_data(&self) -> u32 {
        (self.item_data[0] as u32)
            | ((self.item_data[1] as u32) << 8)
            | ((self.item_data[2] as u32) << 16)
    }
}

/// Full rare drop table for a single section ID.
#[derive(Clone, Debug)]
pub struct RtSet {
    pub enemy_rares: Vec<RtEntry>,
    pub box_rares: Vec<RtEntry>
}
impl Serial for RtSet {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.enemy_rares, 101, dst));
        try!(write_array(&self.box_rares, 30, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let enemy_rares = try!(read_array(101, src));
        let box_rares = try!(read_array(30, src));
        Ok(RtSet {
            enemy_rares: enemy_rares,
            box_rares: box_rares
        })
    }
}

/// Full rare drop table for a full episode.
#[derive(Clone, Debug)]
pub struct ItemRT {
    sections: Vec<RtSet>
}

impl ItemRT {
    pub fn load_from_buffers(files: &[&[u8]]) -> io::Result<ItemRT> {
        if files.len() != 10 {
            return Err(io::Error::new(io::ErrorKind::Other, "Not enough files, need 10"));
        }

        let mut sections = Vec::with_capacity(10);
        for f in files.iter() {
            let mut cursor = Cursor::new(f);
            let section = try!(RtSet::deserialize(&mut cursor));
            sections.push(section);
        }

        Ok(ItemRT {
            sections: sections
        })
    }
}

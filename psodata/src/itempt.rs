//! Structures inside the ItemPT.gsl archive .rel files.
//!
//! ItemPT.gsl contains:
//!
//! ItemPT{episode}{challenge}{difficulty}.rel
//! episode = ["" => 1, "l" => 2, "e" => 4]
//! challenge = ["c" => true, "" => false]
//! difficulty = ["n", "h", "v", "u"]
//!
//! Each file has a `ProbTable` entry for one Section ID in the following
//! order:
//!
//! * Viridia (0)
//! * Greenill (1)
//! * Skyly (2)
//! * Bluefull (3)
//! * Purplenum (4)
//! * Pinkal (5)
//! * Redria (6)
//! * Oran (7)
//! * Yellowboze (8)
//! * Whitill (9)
//!
//! Note that episode 4 data was never actually procured, so specially doctored
//! ItemPT.gsl files have hand-made Ep4 data. Sometimes they don't have proper
//! headers, so we have to make assumptions about where they are in the file.
//!
//! V3 (GC, BB, Xbox?) probability tables are Big Endian even on BB.

use std::io::{Write, Read, Cursor};
use std::io;

use byteorder::{BigEndian as BE, ReadBytesExt};

use psoserial::Serial;
use psoserial::util::*;

#[derive(Clone, Debug)]
pub struct ItemPT {
    sections: Vec<ProbTable>
}

impl ItemPT {
    pub fn load_from_buffers(files: &[&[u8]]) -> io::Result<ItemPT> {
        if files.len() != 10 {
            return Err(io::Error::new(io::ErrorKind::Other, "Not enough files, need 10"));
        }

        let mut sections = Vec::with_capacity(10);
        for f in files.iter() {
            let mut cursor = Cursor::new(f);
            let section = try!(ProbTable::deserialize(&mut cursor));
            sections.push(section);
        }

        Ok(ItemPT {
            sections: sections
        })
    }
}

// We have to circumvent some language limitations at the moment... there's no
// type level integers so implementations for [T; x] only go up to 32. And
// staticvec doesn't suffice because it can't impl Copy. It'd be super nice to
// have Copy.

/// A single entry in a GC/BB probability table.
#[derive(Clone, Debug)]
pub struct ProbTable {
    pub weapon_ratio: [i8; 12],
    pub weapon_minrank: [i8; 12],
    pub weapon_upgfloor: [i8; 12],
    pub power_pattern: [[i8; 4]; 9],
    pub percent_pattern: [[u16; 6]; 23],
    pub area_pattern: [[i8; 10]; 3],
    pub percent_attachment: [[i8; 10]; 6],
    pub element_ranking: [i8; 10],
    pub element_probability: [i8; 10],
    pub armor_ranking: [i8; 5],
    pub slot_ranking: [i8; 5],
    pub unit_level: [i8; 10],
    pub tool_freq: [[u16; 10]; 28],
    pub tech_freq: [[u8; 10]; 19],
    pub tech_levels: [[i8; 10]; 19],
    pub enemy_dar: Vec<i8>, // 100
    pub enemy_meseta: Vec<[u16; 2]>, // 100
    pub enemy_drop: Vec<i8>, // 100
    pub box_meseta: [[u16; 2]; 10],
    pub box_drop: [[u8; 10]; 7],
    pub padding: u16,
    pub pointers: [u32; 18],
    pub armor_level: i32
}

impl Serial for ProbTable {
    fn serialize(&self, _dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<ProbTable> {
        let weapon_ratio = try!(Serial::deserialize(src));
        let weapon_minrank = try!(Serial::deserialize(src));
        let weapon_upgfloor = try!(Serial::deserialize(src));
        let power_pattern = try!(Serial::deserialize(src));
        let percent_pattern = {
            let mut ret: [[u16; 6]; 23] = Default::default();
            for i in 0..23 {
                for k in 0..6 {
                    ret[i][k] = try!(src.read_u16::<BE>());
                }
            }
            ret
        };
        let area_pattern = try!(Serial::deserialize(src));
        let percent_attachment = try!(Serial::deserialize(src));
        let element_ranking = try!(Serial::deserialize(src));
        let element_probability = try!(Serial::deserialize(src));
        let armor_ranking = try!(Serial::deserialize(src));
        let slot_ranking = try!(Serial::deserialize(src));
        let unit_level = try!(Serial::deserialize(src));
        let tool_freq = {
            let mut ret: [[u16; 10]; 28] = Default::default();
            for i in 0..28 {
                for k in 0..10 {
                    ret[i][k] = try!(src.read_u16::<BE>());
                }
            }
            ret
        };
        let tech_freq = try!(Serial::deserialize(src));
        let tech_levels = try!(Serial::deserialize(src));
        let enemy_dar = try!(read_array(100, src));
        let enemy_meseta = {
            let mut ret = Vec::with_capacity(100);
            for _ in 0..100 {
                let mut add: [u16; 2] = Default::default();
                for i in 0..2 {
                    add[i] = try!(src.read_u16::<BE>());
                }
                ret.push(add);
            }
            ret
        };
        let enemy_drop = try!(read_array(100, src));
        let box_meseta = {
            let mut ret: [[u16; 2]; 10] = Default::default();
            for i in 0..10 {
                for k in 0..2 {
                    ret[i][k] = try!(src.read_u16::<BE>());
                }
            }
            ret
        };
        let box_drop = try!(Serial::deserialize(src));
        let padding = try!(src.read_u16::<BE>());
        //let pointers = try!(Serial::deserialize(src));
        let pointers = {
            let mut ret: [u32; 18] = Default::default();
            for i in 0..18 {
                ret[i] = try!(src.read_u32::<BE>());
            }
            ret
        };
        let armor_level = try!(src.read_i32::<BE>());
        Ok(ProbTable {
            weapon_ratio: weapon_ratio,
            weapon_minrank: weapon_minrank,
            weapon_upgfloor: weapon_upgfloor,
            power_pattern: power_pattern,
            percent_pattern: percent_pattern,
            area_pattern: area_pattern,
            percent_attachment: percent_attachment,
            element_ranking: element_ranking,
            element_probability: element_probability,
            armor_ranking: armor_ranking,
            slot_ranking: slot_ranking,
            unit_level: unit_level,
            tool_freq: tool_freq,
            tech_freq: tech_freq,
            tech_levels: tech_levels,
            enemy_dar: enemy_dar,
            enemy_meseta: enemy_meseta,
            enemy_drop: enemy_drop,
            box_meseta: box_meseta,
            box_drop: box_drop,
            padding: padding,
            pointers: pointers,
            armor_level: armor_level
        })
    }
}

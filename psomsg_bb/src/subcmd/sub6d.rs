//! Subcommand 6D has a different structure and is used exclusively for
//! BB-specific messages.

use std::io::{Read, Write};
use std::io;

use psoserial::Serial;
use psoserial::util::*;

use ::chara::{CharStats, Inventory, BankItem};

#[derive(Clone, Debug, Default)]
pub struct Bb6DJoinerChar {
    pub data: Option<Bb6DJoinerCharInner>
}
impl Serial for Bb6DJoinerChar {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        // full character is optional here
        if let Some(ref c) = self.data {
            try!(c.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        use std::io::Cursor;

        let mut buffer = vec![0; 0x4C0];
        match read_exact(src, &mut buffer) {
            Ok(_) => {
                let mut cursor = Cursor::new(buffer);
                let data = try!(Serial::deserialize(&mut cursor));
                Ok(Bb6DJoinerChar {
                    data: Some(data)
                })
            },
            Err(_) => {
                Ok(Bb6DJoinerChar {
                    data: None
                })
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bb6DJoinerCharInner {
    pub unk1: Vec<u8>, // 0x00-0x74    size 0x74
    pub guildcard: u32, // 0x74-0x78
    pub unk2: Vec<u8>, // 0x78-0xBC   size 0x44
    pub techniques: Vec<u8>, // 0xBC-0xD0    size 0x20
    pub unk3: Vec<u8>, // 0xD0-0x108    size 0x38
    pub npccrash_buf: Vec<u8>, // 0x108-0x112    size 10
    pub unk4: Vec<u8>, // 0x112-0x140    size 0x2E
    pub stats: CharStats, // 0x140-0x14e    size 0x14
    pub unk5: Vec<u8>, // 0x14e-0x164    size 0x24
    pub inventory: Inventory // 0x164-0x4B0    size 0x34C
}
impl Serial for Bb6DJoinerCharInner {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.unk1, 0x74, dst));
        try!(self.guildcard.serialize(dst));
        try!(write_array(&self.unk2, 0x44, dst));
        try!(write_array(&self.techniques, 0x20, dst));
        try!(write_array(&self.unk3, 0x38, dst));
        try!(write_array(&self.npccrash_buf, 10, dst));
        try!(write_array(&self.unk4, 0x2E, dst));
        try!(self.stats.serialize(dst));
        try!(write_array(&self.unk5, 0x24, dst));
        try!(self.inventory.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        error!("unk1");
        let unk1 = try!(read_array(0x74, src));
        error!("guildcard");
        let guildcard = try!(Serial::deserialize(src));
        error!("unk2");
        let unk2 = try!(read_array(0x44, src));
        error!("techniques");
        let techniques = try!(read_array(0x20, src));
        error!("unk3");
        let unk3 = try!(read_array(0x38, src));
        error!("npccrash_buf");
        let npccrash_buf = try!(read_array(10, src));
        error!("unk4");
        let unk4 = try!(read_array(0x2E, src));
        error!("stats");
        let stats = try!(Serial::deserialize(src));
        error!("unk5");
        let unk5 = try!(read_array(0x24, src));
        error!("inventory");
        let inventory = try!(Serial::deserialize(src));
        Ok(Bb6DJoinerCharInner {
            unk1: unk1,
            guildcard: guildcard,
            unk2: unk2,
            techniques: techniques,
            unk3: unk3,
            npccrash_buf: npccrash_buf,
            unk4: unk4,
            stats: stats,
            unk5: unk5,
            inventory: inventory
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bb6DBankInv {
    pub checksum: u32,
    // item_count: u32
    pub meseta: u32,
    pub items: Vec<BankItem>
}
impl Serial for Bb6DBankInv {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        // This has a checksum, but the client never verifies it.
        try!(self.checksum.serialize(dst));
        try!((self.items.len() as u32).serialize(dst));
        try!(self.meseta.serialize(dst));
        for i in self.items.iter() {
            try!(i.serialize(dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let checksum = try!(Serial::deserialize(src));
        let item_count = try!(u32::deserialize(src));
        let meseta = try!(Serial::deserialize(src));
        let mut items = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            let item: BankItem = try!(Serial::deserialize(src));
            items.push(item);
        }
        Ok(Bb6DBankInv {
            checksum: checksum,
            meseta: meseta,
            items: items
        })
    }
}

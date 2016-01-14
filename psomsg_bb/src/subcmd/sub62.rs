use std::io::{Write, Read};
use std::io;

use psoserial::Serial;
use psoserial::util::*;

use ::chara::{CharStats, Inventory};

#[derive(Clone, Debug, Default)]
pub struct Bb62PickUpItem {
    pub item_id: u32,
    pub area: u8,
    pub unused1: u8,
    pub unused2: u16
}

impl Serial for Bb62PickUpItem {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.item_id.serialize(dst));
        try!(self.area.serialize(dst));
        try!(self.unused1.serialize(dst));
        try!(self.unused2.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let item_id = try!(Serial::deserialize(src));
        let area = try!(Serial::deserialize(src));
        let unused1 = try!(Serial::deserialize(src));
        let unused2 = try!(Serial::deserialize(src));
        Ok(Bb62PickUpItem {
            item_id: item_id,
            area: area,
            unused1: unused1,
            unused2: unused2
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bb62ItemReq {
    pub area: u8,
    pub pt_index: u8,
    pub req: u16,
    pub x: f32,
    pub y: f32,
    pub unk1: u32,
    pub unk2: u32
}
impl Serial for Bb62ItemReq {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.area.serialize(dst));
        try!(self.pt_index.serialize(dst));
        try!(self.req.serialize(dst));
        try!(self.x.serialize(dst));
        try!(self.y.serialize(dst));
        try!(self.unk1.serialize(dst));
        try!(self.unk2.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let area: u8 = try!(Serial::deserialize(src));
        let pt_index: u8 = try!(Serial::deserialize(src));
        let req: u16 = try!(Serial::deserialize(src));
        let x: f32 = try!(Serial::deserialize(src));
        let y: f32 = try!(Serial::deserialize(src));
        let unk1: u32 = try!(Serial::deserialize(src));
        let unk2: u32 = try!(Serial::deserialize(src));
        Ok(Bb62ItemReq {
            area: area,
            pt_index: pt_index,
            req: req,
            x: x,
            y: y,
            unk1: unk1,
            unk2: unk2
        })
    }
}

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

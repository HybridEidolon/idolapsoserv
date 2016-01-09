use std::io::{Write, Read};
use std::io;

use psoserial::Serial;

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

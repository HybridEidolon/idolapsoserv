use ::util::*;
use ::Serial;

use std::io;
use std::io::{Write, Read};

use byteorder::{LittleEndian as LE, ReadBytesExt};

#[derive(Clone, Debug)]
pub struct BbMiniCharData {
    // uint32_t exp;
    // uint32_t level;
    // char guildcard_str[16];
    // uint32_t unk3[2];                   /* Named to match other structs */
    // uint32_t name_color;
    // uint8_t model;
    // uint8_t unused[15];
    // uint32_t name_color_checksum;
    // uint8_t section;
    // uint8_t ch_class;
    // uint8_t v2flags;
    // uint8_t version;
    // uint32_t v1flags;
    // uint16_t costume;
    // uint16_t skin;
    // uint16_t face;
    // uint16_t head;
    // uint16_t hair;
    // uint16_t hair_r;
    // uint16_t hair_g;
    // uint16_t hair_b;
    // float prop_x;
    // float prop_y;
    // uint16_t name[16];
    // uint32_t play_time;
    pub exp: u32,
    pub level: u32,
    pub guildcard: String,
    pub name_color: u32,
    pub model: u8,
    pub name_color_checksum: u32,
    pub section: u8,
    pub class: u8,
    pub v2flags: u8,
    pub version: u8,
    pub v1flags: u32,
    pub costume: u16,
    pub skin: u16,
    pub face: u16,
    pub head: u16,
    pub hair: u16,
    pub hair_r: u16,
    pub hair_g: u16,
    pub hair_b: u16,
    pub prop_x: f32,
    pub prop_y: f32,
    pub name: String, // 8 characters of some unknown encoding
    pub play_time: u32
}
impl Serial for BbMiniCharData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.exp.serialize(dst));
        try!(self.level.serialize(dst));
        try!(write_ascii_len(&self.guildcard, 16, dst));
        try!(dst.write_all(&[0; 8]));
        try!(self.name_color.serialize(dst));
        try!(self.model.serialize(dst));
        try!(dst.write_all(&[0; 15]));
        try!(self.name_color_checksum.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.class.serialize(dst));
        try!(self.v2flags.serialize(dst));
        try!(self.version.serialize(dst));
        try!(self.v1flags.serialize(dst));
        try!(self.costume.serialize(dst));
        try!(self.skin.serialize(dst));
        try!(self.face.serialize(dst));
        try!(self.head.serialize(dst));
        try!(self.hair.serialize(dst));
        try!(self.hair_r.serialize(dst));
        try!(self.hair_g.serialize(dst));
        try!(self.hair_b.serialize(dst));
        try!(self.prop_x.serialize(dst));
        try!(self.prop_y.serialize(dst));
        try!(write_utf16_len(&self.name, 32, dst));
        try!(self.play_time.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let exp = try!(src.read_u32::<LE>());
        let level = try!(src.read_u32::<LE>());
        let guildcard = try!(read_ascii_len(16, src));
        try!(src.read(&mut [0; 8]));
        let name_color = try!(src.read_u32::<LE>());
        let model = try!(src.read_u8());
        try!(src.read(&mut [0; 15]));
        let name_color_checksum = try!(src.read_u32::<LE>());
        let section = try!(src.read_u8());
        let class = try!(src.read_u8());
        let v2flags = try!(src.read_u8());
        let version = try!(src.read_u8());
        let v1flags = try!(src.read_u32::<LE>());
        let costume = try!(src.read_u16::<LE>());
        let skin = try!(src.read_u16::<LE>());
        let face = try!(src.read_u16::<LE>());
        let head = try!(src.read_u16::<LE>());
        let hair = try!(src.read_u16::<LE>());
        let hair_r = try!(src.read_u16::<LE>());
        let hair_g = try!(src.read_u16::<LE>());
        let hair_b = try!(src.read_u16::<LE>());
        let prop_x = try!(src.read_f32::<LE>());
        let prop_y = try!(src.read_f32::<LE>());
        let name = try!(read_utf16_len(32, src));
        let play_time = try!(src.read_u32::<LE>());
        Ok(BbMiniCharData {
            exp: exp,
            level: level,
            guildcard: guildcard,
            name_color: name_color,
            model: model,
            name_color_checksum: name_color_checksum,
            section: section,
            class: class,
            v2flags: v2flags,
            version: version,
            v1flags: v1flags,
            costume: costume,
            skin: skin,
            face: face,
            head: head,
            hair: hair,
            hair_r: hair_r,
            hair_g: hair_g,
            hair_b: hair_b,
            prop_x: prop_x,
            prop_y: prop_y,
            name: name,
            play_time: play_time
        })
    }
}

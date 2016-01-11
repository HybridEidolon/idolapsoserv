//! Map files for Blue Burst.

use std::io::{Read, Write, Seek, SeekFrom};
use std::io;

use psoserial::Serial;
use psoserial::util::*;

// These corrospond to the maps the client uses. They're simply arrays of
// structures.

#[derive(Clone, Debug)]
pub struct MapEnemy {
    pub base: u32,
    pub reserved1: u16,
    pub num_clones: u16,
    pub reserved2: Vec<u32>,
    pub skin: u32,
    pub reserved3: u32
}

impl Serial for MapEnemy {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.base.serialize(dst));
        try!(self.reserved1.serialize(dst));
        try!(self.num_clones.serialize(dst));
        try!(write_array(&self.reserved2, 14, dst));
        try!(self.skin.serialize(dst));
        try!(self.reserved3.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let base = try!(Serial::deserialize(src));
        let reserved1 = try!(Serial::deserialize(src));
        let num_clones = try!(Serial::deserialize(src));
        let reserved2 = try!(read_array(14, src));
        let skin = try!(Serial::deserialize(src));
        let reserved3 = try!(Serial::deserialize(src));
        Ok(MapEnemy {
            base: base,
            reserved1: reserved1,
            num_clones: num_clones,
            reserved2: reserved2,
            skin: skin,
            reserved3: reserved3
        })
    }
}

impl Default for MapEnemy {
    fn default() -> MapEnemy {
        MapEnemy {
            base: Default::default(),
            reserved1: Default::default(),
            num_clones: Default::default(),
            reserved2: vec![Default::default(); 14],
            skin: Default::default(),
            reserved3: Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapObject {
    pub skin: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub obj_id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rpl: u32,
    pub rotation: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub data: Vec<u32>
}

impl Serial for MapObject {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.skin.serialize(dst));
        try!(self.unk1.serialize(dst));
        try!(self.unk2.serialize(dst));
        try!(self.obj_id.serialize(dst));
        try!(self.x.serialize(dst));
        try!(self.y.serialize(dst));
        try!(self.z.serialize(dst));
        try!(self.rpl.serialize(dst));
        try!(self.rotation.serialize(dst));
        try!(self.unk3.serialize(dst));
        try!(self.unk4.serialize(dst));
        try!(write_array(&self.data, 6, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let skin = try!(Serial::deserialize(src));
        let unk1 = try!(Serial::deserialize(src));
        let unk2 = try!(Serial::deserialize(src));
        let obj_id = try!(Serial::deserialize(src));
        let x = try!(Serial::deserialize(src));
        let y = try!(Serial::deserialize(src));
        let z = try!(Serial::deserialize(src));
        let rpl = try!(Serial::deserialize(src));
        let rotation = try!(Serial::deserialize(src));
        let unk3 = try!(Serial::deserialize(src));
        let unk4 = try!(Serial::deserialize(src));
        let data = try!(read_array(6, src));
        Ok(MapObject {
            skin: skin,
            unk1: unk1,
            unk2: unk2,
            obj_id: obj_id,
            x: x,
            y: y,
            z: z,
            rpl: rpl,
            rotation: rotation,
            unk3: unk3,
            unk4: unk4,
            data: data
        })
    }
}

impl Default for MapObject {
    fn default() -> MapObject {
        MapObject {
            skin: Default::default(),
            unk1: Default::default(),
            unk2: Default::default(),
            obj_id: Default::default(),
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            rpl: Default::default(),
            rotation: Default::default(),
            unk3: Default::default(),
            unk4: Default::default(),
            data: vec![Default::default(); 6]
        }
    }
}

/// Read a `Read` fully, parsing out all enemy structures.
pub fn read_map_enemies<R: Read + Seek>(mut r: R) -> io::Result<Vec<MapEnemy>> {
    try!(r.seek(SeekFrom::Start(0)));
    let size = try!(r.seek(SeekFrom::End(0)));
    let elements = size / 72;
    if size % 72 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Size not a multiple of 72"))
    }
    try!(r.seek(SeekFrom::Start(0)));

    let mut enemies = Vec::new();
    for _ in 0..elements {
        let enemy = try!(MapEnemy::deserialize(&mut r));
        enemies.push(enemy);
    }
    Ok(enemies)
}

/// Read a `Read` fully, parsing out all map object structures.
pub fn read_map_objects<R: Read + Seek>(mut r: R) -> io::Result<Vec<MapObject>> {
    try!(r.seek(SeekFrom::Start(0)));
    let size = try!(r.seek(SeekFrom::End(0)));
    let elements = size / 68;
    if size % 68 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Size not a multiple of 68"))
    }
    try!(r.seek(SeekFrom::Start(0)));

    let mut objects = Vec::new();
    for _ in 0..elements {
        let object = try!(MapObject::deserialize(&mut r));
        objects.push(object);
    }
    Ok(objects)
}

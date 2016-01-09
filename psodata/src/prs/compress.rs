use std::io;
use std::io::Read;

//use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

// struct Ctx<'a> {
//     src: &'a mut Read,
//     dst: Cursor<Vec<u8>>,
// }

pub fn compress(_: &mut Read) -> io::Result<Vec<u8>> {
    unimplemented!()
}

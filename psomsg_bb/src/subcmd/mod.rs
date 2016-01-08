//! Subcommand message types, for message 0x60

use std::io;
use std::io::{Read, Write};

use psoserial::Serial;
use psomsg_common::util::*;

pub mod wrapper;

pub use self::wrapper::BbSubCmd60;

#[derive(Clone, Debug)]
pub struct QuestData1(pub Vec<u8>);
impl Serial for QuestData1 {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.0, 0x0208, dst));
        try!(0u32.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(read_array(0x0208, src));
        try!(u32::deserialize(src));
        Ok(QuestData1(data))
    }
}
impl Default for QuestData1 {
    fn default() -> QuestData1 {
        QuestData1(vec![0u8; 0x0208])
    }
}

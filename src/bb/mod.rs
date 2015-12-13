use std::io;
use std::io::{Read, Cursor};

/// Utility function to read a key table from a Read
pub fn read_key_table(r: &mut Read) -> io::Result<Vec<u32>> {
    let mut data = Vec::with_capacity(1042 * 4);
    try!(r.read_to_end(&mut data));
    let mut key_table: Vec<u32> = Vec::with_capacity(1042);
    let mut cur = Cursor::new(data);
    loop {
        use byteorder::{LittleEndian, ReadBytesExt};
        match cur.read_u32::<LittleEndian>() {
            Err(_) => break,
            Ok(n) => key_table.push(n)
        }
    }
    Ok(key_table)
}

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::io;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

struct DeCtx<'a> {
    src: &'a mut Read,
    dst: Cursor<Vec<u8>>,
    flags: u8,
    bit_pos: u8
}

impl<'a> DeCtx<'a> {
    fn read_bit(&mut self) -> io::Result<bool> {
        if self.bit_pos == 0 {
            self.flags = try!(self.src.read_u8());
            self.bit_pos = 8;
        }

        let ret = self.flags & 1;
        self.flags >>= 1;
        self.bit_pos -= 1;

        match ret { 0 => Ok(false), _ => Ok(true) }
    }
}

#[allow(overflowing_literals)]
pub fn decompress(src: &mut Read) -> io::Result<Vec<u8>> {
    let mut ctx = DeCtx {
        src: src,
        dst: Cursor::new(Vec::new()),
        flags: 0,
        bit_pos: 0
    };

    let mut flag: u32;
    let mut size: u32;
    let mut offset: i32;
    loop {
        // Copy single byte
        match ctx.read_bit() {
            Ok(true) => {
                // copy byte
                try!(ctx.dst.write_u8(try!(ctx.src.read_u8())));
                continue
            },
            Ok(_) => (),
            Err(e) => return Err(e)
        }

        match ctx.read_bit() {
            Ok(true) => {
                // Long copy or end of file
                offset = try!(ctx.src.read_u16::<LE>()) as i32;
                if offset == 0 {
                    // EOF
                    break
                }
                size = (offset & 0x0007) as u32;
                offset = offset.wrapping_shr(3);
                if size == 0 {
                    // Next byte is actual size
                    size = try!(ctx.src.read_u8()) as u32;
                } else {
                    size += 2;
                }

                offset |= 0xFFFFE000;
            },
            Ok(false) => {
                // Short copy
                flag = match try!(ctx.read_bit()) { true => 1, false => 0 };
                size = match try!(ctx.read_bit()) { true => 1, false => 0 };
                size = (size | (flag << 1)) + 2;
                offset = try!(ctx.src.read_u8()) as i32;

                offset |= 0xFFFFFF00;
            }
            Err(e) => return Err(e)
        }

        // Copy duplicate bytes
        for _ in 0..(size+1) {
            try!(ctx.dst.seek(SeekFrom::Current(offset as i64)));
            let b = try!(ctx.dst.read_u8());
            try!(ctx.dst.seek(SeekFrom::Current(-1)));
            try!(ctx.dst.seek(SeekFrom::Current(-offset as i64)));
            try!(ctx.dst.write_u8(b));
        }
    }

    Ok(ctx.dst.into_inner())
}

use std::io::{Read, Write};
use std::io;

use ::Serial;

pub fn read_ascii_len(len: u32, src: &mut Read) -> io::Result<String> {
    use encoding::all::ASCII;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    // up to first null
    let mut end = 0;
    {
        for (i, c) in r.iter().enumerate() {
            if *c == 0 {
                end = i;
                break;
            }
        }
    }
    match ASCII.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode ascii: {:?}", e)))
    }
}

pub fn write_ascii_len(s: &str, len: usize, dst: &mut Write) -> io::Result<()> {
    use encoding::all::ASCII;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match ASCII.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode ASCII: {:?}", e)))
    };

    let padding: isize = len as isize - r.len() as isize;
    if padding < 0 {
        warn!("utf16 string too long, truncating to fit");
        try!(dst.write_all(&r[..len]));
        Ok(())
    } else {
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

pub fn read_utf16(src: &mut Read) -> io::Result<String> {
    use encoding::all::UTF_16LE;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;
    let mut buf = Vec::new();
    // Keep reading until EOF
    loop {
        let mut r_buf = [0u8; 2];
        // God damn it
        // This will only work if the stream ends (i.e. using a Cursor)
        // It will block on a raw network stream...
        let bytes_read = try!(src.read(&mut r_buf[..]));
        if bytes_read != 2 {
            break;
        }
        if r_buf[0] == 0 && r_buf[1] == 0 {
            break;
        }
        buf.push(r_buf[0]);
        buf.push(r_buf[1]);
    }

    let r = match UTF_16LE.decode(&buf[..], Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to decode utf16: {:?}", e)))
    };
    Ok(r)
}

pub fn write_utf16(s: &str, dst: &mut Write) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let mut r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };
    r.push(0);
    r.push(0);
    try!(dst.write_all(&r[..]));
    Ok(())
}

pub fn read_utf16_len(len: usize, src: &mut Read) -> io::Result<String> {
    use encoding::all::UTF_16LE;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    // up to first 2 nulls
    let mut end = 0;
    {
        for i in 0..(r.len()/2) {
            if r[(i*2)] == 0 && r[(i*2)+1] == 0 {
                end = i*2;
            }
        }
    }
    match UTF_16LE.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode utf16: {:?}", e)))
    }
}

pub fn write_utf16_len(s: &str, len: usize, dst: &mut Write) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };

    let padding: isize = len as isize - r.len() as isize;
    if padding < 0 {
        warn!("utf16 string too long, truncating to fit");
        try!(dst.write_all(&r[..len]));
        Ok(())
    } else {
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

pub fn read_array<T: Serial + Default>(len: u32, src: &mut Read) -> io::Result<Vec<T>> {
    let mut r = Vec::with_capacity(len as usize);
    for _ in 0..len {
        r.push(try!(T::deserialize(src)));
    }
    Ok(r)
}

pub fn write_array<T: Serial + Default>(sl: &[T], len: u32, dst: &mut Write) -> io::Result<()> {
    if sl.len() > len as usize {
        warn!("Slice is larger than desired length, writing truncated");
        for i in sl.iter().take(len as usize) {
            try!(i.serialize(dst));
        }
        Ok(())
    } else {
        let padding = len as i32 - sl.len() as i32;
        for i in sl.iter() {
            try!(i.serialize(dst));
        }
        for _ in 0..padding {
            try!(T::default().serialize(dst));
        }
        Ok(())
    }
}

/// Until `feature(read_exact)` is stabilized, this is a direct copy of Read::read_exact from the
/// standard library. `read_exact` will be stable in 1.6.0
pub fn read_exact(read: &mut Read, mut buf: &mut [u8]) -> io::Result<()> {
    while !buf.is_empty() {
        match read.read(buf) {
            Ok(0) => break,
            Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    if !buf.is_empty() {
        Err(io::Error::new(io::ErrorKind::Other,
                       "failed to fill whole buffer"))
    } else {
        Ok(())
    }
}

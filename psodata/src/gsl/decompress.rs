//! Unarchive routine for GSL format buffers.

use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::ascii::AsciiExt;

use byteorder::{LittleEndian as LE, BigEndian as BE, ReadBytesExt};

use super::{GslHeader, GslFile};

fn get_seek_size<S: Seek>(mut src: S) -> io::Result<u64> {
    let current = try!(src.seek(SeekFrom::Current(0)));
    try!(src.seek(SeekFrom::Start(0)));
    let end = try!(src.seek(SeekFrom::End(0)));
    try!(src.seek(SeekFrom::Start(current)));
    Ok(end)
}

fn decompress<R: Read + Seek>(mut src: R, big_endian: Option<bool>) -> io::Result<Vec<GslFile>> {
    // Figure out archive size. Used in endianness guessing later.
    let archive_size = try!(get_seek_size(&mut src));

    debug!("Unzipping GSL archive of size {}", archive_size);

    // Read the first file
    let mut headers = Vec::new();
    match read_file_hdr(&mut src, archive_size, big_endian) {
        Ok(Some(fh)) => {
            headers.push(fh);
        },
        Ok(None) => {
            return Err(io::Error::new(io::ErrorKind::Other, "GSL archive has no files"))
        },
        Err(e) => {
            return Err(e)
        }
    }

    loop {
        match try!(read_file_hdr(&mut src, archive_size, big_endian)) {
            Some(fh) => headers.push(fh),
            None => break
        }
    }

    // we now have all the file headers, load the files themselves

    let mut files = Vec::new();
    for h in headers {
        try!(src.seek(SeekFrom::Start(h.offset as u64)));
        let mut data = vec![0; h.size as usize];
        try!(read_exact(&mut src, &mut data));
        files.push(GslFile {
            name: h.name,
            data: data
        });
    }

    Ok(files)
}

/// De-archives a stream and returns the GSL archive with buffers for each file.
/// Use with Little Endian archives (Blue Burst).
pub fn decompress_le<R: Read + Seek>(src: R) -> io::Result<Vec<GslFile>> {
    decompress(src, Some(false))
}

/// De-archives a stream and returns the GSL archive with buffers for each file.
/// Use with Big Endian archives (GameCube).
pub fn decompress_be<R: Read + Seek>(src: R) -> io::Result<Vec<GslFile>> {
    decompress(src, Some(true))
}

/// De-archives a stream and returns the GSL archive with buffers for each file.
/// Will guess the endianness of the file.
///
/// # Errors
/// The heuristic used to guess endiannness fails under certain circumstances.
/// Thus, the archive may fail to parse with an `Err` result.
pub fn decompress_guess<R: Read + Seek>(src: R) -> io::Result<Vec<GslFile>> {
    decompress(src, None)
}

fn read_file_hdr<R: Read + Seek>(mut src: R, arch_size: u64, big_endian: Option<bool>) -> io::Result<Option<GslHeader>> {
    let mut fn_buf = vec![0; 32];
    try!(read_exact(&mut src, &mut fn_buf));

    if fn_buf[0] == 0 {
        // end of file headers.
        return Ok(None)
    }
    if !fn_buf.is_ascii() {
        return Err(io::Error::new(io::ErrorKind::Other, "file name is not valid ASCII"))
    }

    let mut filename = String::new();
    for c in fn_buf.iter() {
        if *c == 0 {
            break
        }
        filename.push(*c as char);
    }

    // GameCube is big endian, Blue Burst is Windows so little endian
    let offset;
    let size;
    match big_endian {
        Some(true) => {
            offset = try!(src.read_u32::<BE>());
            size = try!(src.read_u32::<BE>());
        },
        Some(false) => {
            offset = try!(src.read_u32::<LE>());
            size = try!(src.read_u32::<LE>());
        },
        None => {
            // we have to guess
            let offset_guess = try!(src.read_u32::<BE>());
            let size_guess = try!(src.read_u32::<BE>());

            if offset_guess as u64 > arch_size
                || offset_guess as u64 * 2048 > arch_size
                || size_guess as u64 > arch_size
            {
                debug!("Guessed LittleEndian");
                try!(src.seek(SeekFrom::Current(-8)));
                offset = try!(src.read_u32::<LE>());
                size = try!(src.read_u32::<LE>());
            } else {
                debug!("Guessed BigEndian");
                offset = offset_guess;
                size = size_guess;
            }
        }
    }
    try!(src.seek(SeekFrom::Current(8)));

    let ret = GslHeader {
        name: filename,
        offset: offset * 2048,
        size: size
    };

    debug!("Decoded file header {:?}", ret);

    Ok(Some(ret))
}

// TODO remove this when it is stabilized in Rust 1.6 (1.7?)
fn read_exact<R: Read>(this: &mut R, mut buf: &mut [u8]) -> io::Result<()> {
    while !buf.is_empty() {
        match this.read(buf) {
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

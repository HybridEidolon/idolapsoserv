use std::io::{Cursor, Read};
use std::io;

use psoserial::Serial;
use psomsg::util::*;

static COLOR_RED: &'static str = "\x1B[31m";
static COLOR_RESET: &'static str = "\x1B[0m";

/// Creates a 3-column view of the buffer with index, bytes, and ASCII
/// representation.
pub fn hex_view(buf: &[u8]) -> String {
    let rows = buf.len() / 16 + { if buf.len() % 16 > 0 {1} else {0} };
    let mut output = String::new();
    for row in 0..rows {
        // write the index
        output.push_str(&format!("{:08X} | ", row * 16));

        // write the next 16 bytes
        let leftover;
        let end = {
            if buf.len() > row * 16 + 16 {
                leftover = 0;
                row * 16 + 16
            } else {
                leftover = row * 16 + 16 - buf.len();
                buf.len()
            }
        };

        for b in &buf[(row * 16)..end] {
            output.push_str(&format!("{:02X} ", b))
        }
        for _ in 0..leftover {
            output.push_str("   ");
        }

        output.push_str("| ");
        // write the ascii representation
        for b in &buf[(row * 16)..end] {
            if *b > 31u8 && *b < 127u8 {
                output.push(*b as char);
            } else {
                output.push('.');
            }
        }
        // new line
        output.push('\n');
    }

    output
}

pub fn hex_view_serial<S: Serial>(s: &S) -> String {
    let mut cursor = Cursor::new(Vec::new());
    s.serialize(&mut cursor).unwrap();
    let array = cursor.into_inner();
    hex_view(&array)
}

/// Shows the serialized hex view of the first argument, with different bytes
/// in ANSI escaped red.
pub fn hex_view_diff<S: Serial>(s: &S, buf: &[u8]) -> String {
    let mut cursor = Cursor::new(Vec::new());
    s.serialize(&mut cursor).unwrap();
    let array = cursor.into_inner();

    let rows = array.len() / 16 + { if array.len() % 16 > 0 {1} else {0} };
    let mut output = String::new();
    for row in 0..rows {
        // First row is Serial version
        // write the index
        output.push_str(&format!("{:08X} | ", row * 16));

        // write the next 16 bytes
        let leftover;
        let end = {
            if array.len() > row * 16 + 16 {
                leftover = 0;
                row * 16 + 16
            } else {
                leftover = row * 16 + 16 - array.len();
                array.len()
            }
        };

        for i in (row * 16)..end {
            if (buf.len() > i && buf[i] != array[i]) || buf.len() <= i {
                output.push_str(&format!("{}{:02X}{} ", COLOR_RED, array[i], COLOR_RESET));
            } else {
                output.push_str(&format!("{:02X} ", array[i]));
            }
        }
        // for b in &buf[(row * 16)..end] {
        //     output.push_str(&format!("{:02X} ", b))
        // }
        for _ in 0..leftover {
            output.push_str("   ");
        }

        output.push_str("| ");
        // write the ascii representation
        for b in &array[(row * 16)..end] {
            if *b > 31u8 && *b < 127u8 {
                output.push(*b as char);
            } else {
                output.push('.');
            }
        }

        output.push('\n');
    }

    output
}

/// Reads a raw BB message buffer
pub fn read_bb_msg(src: &mut Read) -> io::Result<Vec<u8>> {
    let mut hdr = vec![0u8; 8];
    debug!("reading header...");
    try!(read_exact(src, &mut hdr));
    let size = hdr[0] as usize + ((hdr[1] as usize) << 8);
    debug!("msg size: {}", size);
    let padding = if size % 8 == 0 { 0 } else { 8 - (size % 8) };
    let mut body = vec![0u8; size + padding - 8];
    debug!("reading body...");
    try!(read_exact(src, &mut body));
    hdr.append(&mut body);
    Ok(hdr)
}

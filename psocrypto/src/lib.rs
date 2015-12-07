extern crate byteorder;
extern crate encoding;

use std::io::{Read, Write};
use std::io;

/// PC crypto. Used by patch server on BB as well. The Dreamcast and
/// PC versions share this crypto.
pub mod pc;

/// Blue Burst-specific crypto. A lot more complex than PC and GC.
pub mod bb;

/// Gamecube games' crypto. PSO Episodes 1 & 2 and Episode 3 use
/// this crypto algorithm.
pub mod gc;

pub use self::bb::BbCipher;
pub use self::pc::PcCipher;

pub trait Encryptor {
    fn encrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String>;
}

pub trait Decryptor {
    fn decrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String>;
}

pub struct DecryptReader<R: Read, D: Decryptor> {
    r: R,
    d: D
}

impl<R: Read, D: Decryptor> DecryptReader<R, D> {
    pub fn new(read: R, decrypt: D) -> Self {
        DecryptReader {
            r: read,
            d: decrypt
        }
    }
}

impl<R: Read, D: Decryptor> Read for DecryptReader<R, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut int_buf = vec![0u8; buf.len()];
        let bytes_read = try!(self.r.read(&mut int_buf[..]));

        // decrypt into output buffer
        if let Err(e) = self.d.decrypt(&int_buf[..], &mut buf[..bytes_read]) {
            return Err(io::Error::new(io::ErrorKind::Other, e))
        }

        Ok(bytes_read)
    }
}

pub struct EncryptWriter<W: Write, E: Encryptor> {
    w: W,
    e: E
}

impl<W: Write, E: Encryptor> EncryptWriter<W, E> {
    pub fn new(write: W, encrypt: E) -> Self {
        EncryptWriter {
            w: write,
            e: encrypt
        }
    }
}

impl<W: Write, E: Encryptor> Write for EncryptWriter<W, E> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut int_buf = vec![0u8; buf.len()];

        // encrypt into int buffer
        if let Err(e) = self.e.encrypt(buf, &mut int_buf[..]) {
            return Err(io::Error::new(io::ErrorKind::Other, e))
        }

        try!(self.w.write_all(&int_buf[..]));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

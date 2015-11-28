use crypto::symmetriccipher::{Decryptor, Encryptor, SynchronousStreamCipher, SymmetricCipherError};
use crypto::buffer::{BufferResult, RefReadBuffer, RefWriteBuffer, ReadBuffer, WriteBuffer};

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// A struct for encrypting and decrypting using the PC and Dreamcast
/// cryptography algorithm.
pub struct PcCipher {
    keys: Vec<u32>,
    pos: usize,
    seed: u32
}

impl PcCipher {
    /// Create a new cipher from the given seed.
    pub fn new(seed: u32) -> Self {
        PcCipher {
            keys: gen_keys(seed),
            pos: 56,
            seed: seed
        }
    }

    fn get_next_key(&mut self) -> u32 {
        let re: u32;
        if self.pos == 56 {
            mix_keys(&mut self.keys);
            self.pos = 1;
        }
        re = self.keys[self.pos];
        self.pos += 1;
        re
    }

    pub fn seed(&self) -> u32 { self.seed }
}

impl Encryptor for PcCipher {
    fn encrypt(&mut self, input: &mut RefReadBuffer, output: &mut RefWriteBuffer, _: bool)
    -> Result<BufferResult, SymmetricCipherError> {
        use std::cmp;
        let count = cmp::min(input.remaining(), output.remaining());
        self.process(input.take_next(count), output.take_next(count));
        if input.is_empty() {
            Ok(BufferResult::BufferOverflow)
        } else {
            Ok(BufferResult::BufferOverflow)
        }
    }
}

impl Decryptor for PcCipher {
    fn decrypt(&mut self, input: &mut RefReadBuffer, output: &mut RefWriteBuffer, eof: bool)
    -> Result<BufferResult, SymmetricCipherError> {
        self.encrypt(input, output, eof)
    }
}

impl SynchronousStreamCipher for PcCipher {
    fn process(&mut self, input: &[u8], output: &mut [u8]) {
        let mut ci = Cursor::new(input);
        let mut co = Cursor::new(output);
        loop {
            if let Ok(num) = ci.read_u32::<LittleEndian>() {
                if let Err(_) = co.write_u32::<LittleEndian>(num ^ self.get_next_key()) {
                    break
                }
            } else {
                break
            }
        }
    }
}

#[inline]
fn mix_keys(keys: &mut [u32]) {
    use std::num::Wrapping as W;
    let mut esi: W<u32>;
    let mut edi: W<u32>;
    let mut eax: W<u32>;
    let mut ebp: W<u32>;
    let mut edx: W<u32>;

    edi = W(1);
    edx = W(0x18);
    eax = edi;
    while edx > W(0) {
        esi = W(keys[(eax + W(0x1F)).0 as usize]);
        ebp = W(keys[eax.0 as usize]);
        ebp = ebp - esi;
        keys[eax.0 as usize] = ebp.0;
        eax = eax + W(1);
        edx = edx - W(1);
    }
    edi = W(0x19);
    edx = W(0x1F);
    eax = edi;
    while edx > W(0) {
        esi = W(keys[(eax - W(0x18)).0 as usize]);
        ebp = W(keys[eax.0 as usize]);
        ebp = ebp - esi;
        keys[eax.0 as usize] = ebp.0;
        eax = eax + W(1);
        edx = edx - W(1);
    }
}

/// Generates PC encryption keys from a seed. Use position 56 when initializing.
fn gen_keys(seed: u32) -> Vec<u32> {
    use std::num::Wrapping as W;

    let mut keys = vec![0; 57];
    let mut esi: W<u32>;
    let mut ebx: W<u32>;
    let mut edi: W<u32>;
    let mut eax: W<u32>;
    let mut edx: W<u32>;
    let mut var1: W<u32>;

    esi = W(1);
    ebx = W(seed);
    edi = W(0x15);
    keys[56] = ebx.0;
    keys[55] = ebx.0;
    while edi <= W(0x46E) {
        eax = edi;
        var1 = eax / W(55);
        edx = eax - (var1 * W(55));
        ebx = ebx - esi;
        edi = edi + W(0x15);
        keys[edx.0 as usize] = esi.0;
        esi = ebx;
        ebx = W(keys[edx.0 as usize]);
    }

    for _ in 0..4 {
        mix_keys(&mut keys);
    };

    keys
}

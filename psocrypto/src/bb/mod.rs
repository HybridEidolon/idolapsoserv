use std::io::Cursor;

use ::{Decryptor, Encryptor};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// A cipher struct for encrypting and decrypting data using Blue Burst's
/// encryption algorithm. This is not a stream cipher.
pub struct BbCipher {
    keys: Vec<u32>,
    seed: Vec<u8>
}

impl BbCipher {
    /// Create a new BbCipher with the given seed and key table.
    ///
    /// The table is a random hash of u32 values that is stored inside the PSOBB
    /// client. In the source of this project, the one used for the Tethealla
    /// EXEs is at data/crypto/bb_table.bin. This must be exactly 1042 u32s long,
    /// or 4168 bytes; the table argument should also be 1042 u32s long.
    ///
    /// # Panics
    ///
    /// This function will panic if table is not 1042 elements long, or if seed is
    /// not 48 elements long.
    pub fn new(seed: &[u8], table: &[u32]) -> Self {
        assert_eq!(table.len(), 1042);

        BbCipher {
            keys: setup_cryptosetup_bb(seed, table),
            seed: seed.to_vec()
        }
    }

    /// Get an immutable view of the seed used to generate the keys.
    pub fn get_seed<'a>(&'a self) -> &'a[u8] {
        &self.seed
    }

    pub fn get_keys<'a>(&'a self) -> &'a[u32] {
        &self.keys
    }
}

impl Encryptor for BbCipher {
    fn encrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String> {
        use std::num::Wrapping as W;
        let mut ebx: W<u32>;
        let mut ebp: W<u32>;
        let mut esi: W<u32>;
        let mut edi: W<u32>;
        let mut tmp: W<u32>;

        if input.len() > output.len() {
            return Err(format!("Buffer size too large in encrypt: input {}, output {}", input.len(), output.len()))
        }

        let mut ci = Cursor::new(input);
        let mut co = Cursor::new(output);


        // Operate on 2 u32 at a time
        loop {
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                ebx = W(n)
            } else { return Ok(()) }
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                tmp = W(n)
            } else { return Err(format!("Invalid length: expected 8 bytes, only read 4")) }

            ebx = ebx ^ W(self.keys[0]);
            ebp = {
                let a = W(self.keys[((ebx >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebx >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebx >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebx & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            ebp = ebp ^ W(self.keys[1]);
            ebp = ebp ^ tmp;

            edi = {
                let a = W(self.keys[((ebp >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebp >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebp >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebp & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            edi = edi ^ W(self.keys[2]);
            ebx = ebx ^ edi;
            esi = {
                let a = W(self.keys[((ebx >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebx >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebx >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebx & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            ebp = ebp ^ esi ^ W(self.keys[3]);
            edi = {
                let a = W(self.keys[((ebp >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebp >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebp >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebp & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            edi = edi ^ W(self.keys[4]);
            ebp = ebp ^ W(self.keys[5]);
            ebx = ebx ^ edi;

            // Write phase
            if let Err(_) = co.write_u32::<LittleEndian>(ebp.0) {
                return Err("Output buffer is not big enough (need 8 more bytes)".to_string())
            }
            if let Err(_) = co.write_u32::<LittleEndian>(ebx.0) {
                return Err("Output buffer is not big enough (need 4 more bytes)".to_string())
            }
        }
    }
}

impl Decryptor for BbCipher {
    fn decrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String> {
        use std::num::Wrapping as W;
        let mut ebx: W<u32>;
        let mut ebp: W<u32>;
        let mut esi: W<u32>;
        let mut edi: W<u32>;
        let mut tmp: W<u32>;

        if input.len() > output.len() {
            return Err(format!("Buffer size too large in decrypt: input {}, output {}", input.len(), output.len()))
        }

        let mut ci = Cursor::new(input);
        let mut co = Cursor::new(output);

        // Operate on 2 u32 at a time
        loop {
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                ebx = W(n)
            } else { return Ok(()) }
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                tmp = W(n)
            } else { return Err(format!("Invalid length: expected 8 bytes, only read 4")) }

            ebx = ebx ^ W(self.keys[5]);
            ebp = {
                let a = W(self.keys[((ebx >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebx >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebx >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebx & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            ebp = ebp ^ W(self.keys[4]);
            ebp = ebp ^ tmp;

            edi = {
                let a = W(self.keys[((ebp >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebp >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebp >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebp & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            edi = edi ^ W(self.keys[3]);
            ebx = ebx ^ edi;
            esi = {
                let a = W(self.keys[((ebx >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebx >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebx >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebx & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            ebp = ebp ^ esi ^ W(self.keys[2]);
            edi = {
                let a = W(self.keys[((ebp >> 0x18) + W(0x12)).0 as usize]);
                let b = W(self.keys[(((ebp >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]);
                let c = W(self.keys[(((ebp >> 0x8) & W(0xFF)) + W(0x212)).0 as usize]);
                let d = W(self.keys[((ebp & W(0xFF)) + W(0x312)).0 as usize]);

                ((a + b) ^ c) + d
            };
            edi = edi ^ W(self.keys[1]);
            ebp = ebp ^ W(self.keys[0]);
            ebx = ebx ^ edi;

            // Write phase
            if let Err(_) = co.write_u32::<LittleEndian>(ebp.0) {
                return Err("Output buffer is not big enough (need 8 more bytes)".to_string())
            }
            if let Err(_) = co.write_u32::<LittleEndian>(ebx.0) {
                return Err("Output buffer is not big enough (need 4 more bytes)".to_string())
            }
        }
    }
}

/// Generates the key vector for Blue Burst encryption.
/// Taken from Sylverant. Holy hell, I'm pretty sure this is just verbatim disassembly.
///
/// Keys should already be populated with a binary key table from the client. It will
/// be changed with the seed.
///
/// This is quite possibly the worst Rust code ever written.
pub fn setup_cryptosetup_bb(salt: &[u8], bbtable: &[u32]) -> Vec<u32> {
    use std::num::Wrapping as W;
    let mut eax: W<u32>;
    let mut ecx: W<u32>;
    let mut edx: W<u32>;
    let mut ebx: W<u32>;
    let mut ebp: W<u32>;
    let mut esi: W<u32>;
    let mut edi: W<u32>;
    let mut ou: W<u32>;

    let mut keys = vec![0u32; 1042];

    assert_eq!(48, salt.len());
    assert_eq!(1042, keys.len());
    assert_eq!(1042, bbtable.len());
    let mut s: Vec<W<u8>> = salt.iter().map(|b| W(*b)).collect();

    // L_CRYPT_BB_InitKey
    {
        let mut i = 0;
        // We could replace this with .step_by once that becomes stable.
        while i < 48 {
            s[i] = s[i] ^ W(0x19u8);
            s[i + 1] = s[i + 1] ^ W(0x16u8);
            s[i + 2] = s[i + 2] ^ W(0x18u8);
            i += 3;
        }
    }

    // this is some hot garbage
    unsafe {
        use std::mem;

        let pcryp: *mut u16 = mem::transmute(&mut keys[0]);
        let bbtbl: *const u16 = mem::transmute(&bbtable[0]);
        let mut eax: u16 = 0;
        let mut ebx: u16 = 0;
        let mut dx: u16;
        for _ in 0..0x12 {
            dx = *bbtbl.offset(eax as isize);
            eax += 1;
            dx = ((dx & 0xFF) << 8) + (dx >> 8);
            *pcryp.offset(ebx as isize) = dx;
            dx = *bbtbl.offset(eax as isize);
            eax += 1;
            dx ^= *pcryp.offset(ebx as isize);
            ebx += 1;
            *pcryp.offset(ebx as isize) = dx;
            ebx += 1;
        }
    }
    // our saved key binaries should already have this set up. (SEGA BB keys AND teth)
    for i in 18..(1024 + 18) {
        keys[i] = bbtable[i];
    }

    ecx = W(0);
    ebx = W(0);

    while ebx < W(0x12) {
        ebp = W(s[ecx.0 as usize].0 as u32) << 0x18;
        eax = ecx + W(1);
        edx = eax - ((eax / W(48)) * W(48));
        eax = (W(s[edx.0 as usize].0 as u32) << 0x10) & W(0xFF0000);
        ebp = (ebp | eax) & W(0xFFFF00FF);
        eax = ecx + W(2);
        edx = eax - ((eax / W(48)) * W(48));
        eax = (W(s[edx.0 as usize].0 as u32) << 0x8) & W(0xFF00);
        ebp = (ebp | eax) & W(0xFFFFFF00);
        eax = ecx + W(3);
        ecx = ecx + W(4);
        edx = eax - ((eax / W(48)) * W(48));
        eax = W(s[edx.0 as usize].0 as u32);
        ebp = ebp | eax;
        eax = ecx;
        edx = eax - ((eax / W(48)) * W(48));
        keys[ebx.0 as usize] = (W(keys[ebx.0 as usize]) ^ ebp).0;
        ecx = edx;
        ebx = ebx + W(1);
    }

    //ebp = W(0);
    esi = W(0);
    ecx = W(0);
    edi = W(0);
    //ebx = W(0);
    edx = W(0x48);

    while edi < edx {
        esi = esi ^ W(keys[0]);
        eax = esi >> 0x18;
        ebx = (esi >> 0x10) & W(0xFF);
        eax = W(keys[(eax + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
        ebx = (esi >> 8) & W(0xFF);
        eax = eax ^ W(keys[(ebx + W(0x212)).0 as usize]);
        ebx = esi & W(0xFF);
        eax = eax + W(keys[(ebx + W(0x312)).0 as usize]);

        eax = eax ^ W(keys[1]);
        ecx = ecx ^ eax;
        ebx = ecx >> 0x18;
        eax = (ecx >> 0x10) & W(0xFF);
        ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
        eax = (ecx >> 8) & W(0xFF);
        ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
        eax = ecx & W(0xFF);
        ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);

        for index in 0..6 {
            ebx = ebx ^ W(keys[(index*2)+2]);
            esi = esi ^ ebx;
            ebx = esi >> 0x18;
            eax = (esi >> 0x10) & W(0xFF);
            ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
            eax = (esi >> 8) & W(0xFF);
            ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
            eax = esi & W(0xFF);
            ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);

            ebx=ebx ^ W(keys[(index*2)+3]);
            ecx = ecx ^ ebx;
            ebx = ecx >> 0x18;
            eax = (ecx >> 0x10) & W(0xFF);
            ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
            eax = (ecx >> 8) & W(0xFF);
            ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
            eax = ecx & W(0xFF);
            ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);
        }

        ebx = ebx ^ W(keys[14]);
        esi = esi ^ ebx;
        eax = esi >> 0x18;
        ebx = (esi >> 0x10) & W(0xFF);
        eax = W(keys[(eax + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
        ebx = (esi >> 8) & W(0xFF);
        eax = eax ^ W(keys[(ebx + W(0x212)).0 as usize]);
        ebx = esi & W(0xFF);
        eax = eax + W(keys[(ebx + W(0x312)).0 as usize]);

        eax = eax ^ W(keys[15]);
        eax = ecx ^ eax;
        ecx = eax >> 0x18;
        ebx = (eax >> 0x10) & W(0xFF);
        ecx = W(keys[(ecx + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
        ebx = (eax >> 8) & W(0xFF);
        ecx = ecx ^ W(keys[(ebx + W(0x212)).0 as usize]);
        ebx = eax & W(0xFF);
        ecx = ecx + W(keys[(ebx + W(0x312)).0 as usize]);

        ecx = ecx ^ W(keys[16]);
        ecx = ecx ^ esi;
        esi = W(keys[17]);
        esi = esi ^ eax;
        keys[(edi / W(4)).0 as usize] = esi.0;
        keys[((edi / W(4)) + W(1)).0 as usize] = ecx.0;
        edi = edi + W(8);
    }

    //eax = W(0);
    //edx = W(0);

    ou = W(0);
    while ou < W(0x1000) {
        edi = W(0x48);
        edx = W(0x448);

        while edi < edx {
            esi = esi ^ W(keys[0]);
            eax = esi >> 0x18;
            ebx = (esi >> 0x10) & W(0xFF);
            eax = W(keys[(eax + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
            ebx = (esi >> 8) & W(0xFF);
            eax = eax ^ W(keys[(ebx + W(0x212)).0 as usize]);
            ebx = esi & W(0xFF);
            eax = eax + W(keys[(ebx + W(0x312)).0 as usize]);

            eax = eax ^ W(keys[1]);
            ecx = ecx ^ eax;
            ebx = ecx >> 0x18;
            eax = (ecx >> 0x10) & W(0xFF);
            ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
            eax = (ecx >> 8) & W(0xFF);
            ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
            eax = ecx & W(0xFF);
            ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);

            for index in 0..6 {
                ebx = ebx ^ W(keys[(index*2)+2]);
                esi = esi ^ ebx;
                ebx = esi >> 0x18;
                eax = (esi >> 0x10) & W(0xFF);
                ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
                eax = (esi >> 8) & W(0xFF);
                ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
                eax = esi & W(0xFF);
                ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);

                ebx = ebx ^ W(keys[(index*2)+3]);
                ecx = ecx ^ ebx;
                ebx = ecx >> 0x18;
                eax = (ecx >> 0x10) & W(0xFF);
                ebx = W(keys[(ebx + W(0x12)).0 as usize]) + W(keys[(eax + W(0x112)).0 as usize]);
                eax = (ecx >> 8) & W(0xFF);
                ebx = ebx ^ W(keys[(eax + W(0x212)).0 as usize]);
                eax = ecx & W(0xFF);
                ebx = ebx + W(keys[(eax + W(0x312)).0 as usize]);
            }

            ebx = ebx ^ W(keys[14]);
            esi = esi ^ ebx;
            eax = esi >> 0x18;
            ebx = (esi >> 0x10) & W(0xFF);
            eax = W(keys[(eax + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
            ebx = (esi >> 8) & W(0xFF);
            eax = eax ^ W(keys[(ebx + W(0x212)).0 as usize]);
            ebx = esi & W(0xFF);
            eax = eax + W(keys[(ebx + W(0x312)).0 as usize]);

            eax = eax ^ W(keys[15]);
            eax = ecx ^ eax;
            ecx = eax >> 0x18;
            ebx = (eax >> 0x10) & W(0xFF);
            ecx = W(keys[(ecx + W(0x12)).0 as usize]) + W(keys[(ebx + W(0x112)).0 as usize]);
            ebx = (eax >> 8) & W(0xFF);
            ecx = ecx ^ W(keys[(ebx + W(0x212)).0 as usize]);
            ebx = eax & W(0xFF);
            ecx = ecx + W(keys[(ebx + W(0x312)).0 as usize]);

            ecx = ecx ^ W(keys[16]);
            ecx = ecx ^ esi;
            esi = W(keys[17]);
            esi = esi ^ eax;
            keys[((ou / W(4)) + (edi / W(4))).0 as usize] = esi.0;
            keys[((ou / W(4)) + (edi / W(4)) + W(1)).0 as usize] = ecx.0;
            edi = edi + W(8);
        }
        ou = ou + W(0x400);
    };
    keys
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use super::BbCipher;
    use ::Encryptor;
    use ::Decryptor;
    use std::io;
    use std::io::{Cursor, Read};
    use byteorder::*;

    fn read_key_table(mut f: File) -> io::Result<Vec<u32>> {

        // let metadata = try!(f.metadata());
        // if metadata.len() != 1042 * 4 {
        //     return Err(io::Error::new(io::ErrorKind::Other, "key table is not correct size"))
        // }

        //drop(metadata);
        let mut data = Vec::with_capacity(1042 * 4);
        try!(f.read_to_end(&mut data));
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

    #[test]
    fn test_cipher_symmetry() {
        let table = read_key_table(File::open("test/bb_table.bin").unwrap()).unwrap();
        let mut cipher = BbCipher::new(&vec![0u8; 48], &table);

        let mut in_buffer: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let expect = in_buffer.clone();
        let mut out_buffer = vec![0u8; 8];

        cipher.encrypt(&in_buffer, &mut out_buffer).unwrap();
        cipher.decrypt(&out_buffer, &mut in_buffer).unwrap();
        assert_eq!(in_buffer, expect);
    }

    #[test]
    fn test_real_header() {
        let incoming: Vec<u8> = vec![179, 239, 115, 210, 246, 22, 169, 122];
        let expected: Vec<u8> = vec![180, 0, 0x93, 0];

        let table = read_key_table(File::open("test/bb_table.bin").unwrap()).unwrap();
        let mut cipher = BbCipher::new(&vec![0u8; 48], &table);
        let mut decrypted = vec![0u8; 8];

        cipher.decrypt(&incoming, &mut decrypted).unwrap();
        assert_eq!(decrypted[0..4], expected[0..4]);
    }

    #[test]
    fn test_zero_seed_cipher_gen() {
        let table = read_key_table(File::open("test/bb_table.bin").unwrap()).unwrap();
        let zero_table = read_key_table(File::open("test/zero_table.bin").unwrap()).unwrap();

        let cipher = BbCipher::new(&vec![0u8; 48], &table);
        assert_eq!(cipher.get_keys().len(), zero_table.len());
        assert_eq!(cipher.get_keys()[..], zero_table[..]);
    }
}

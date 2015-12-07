use std::io::Cursor;

use ::{Decryptor, Encryptor};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

mod table;

pub use self::table::BB_TABLE;

/// A cipher struct for encrypting and decrypting data using Blue Burst's
/// encryption algorithm. This is not a stream cipher.
pub struct BbCipher {
    keys: Vec<u32>,
    seed: Vec<u8>
}

impl BbCipher {
    /// Create a new BbCipher with the given seed.
    pub fn new(seed: &[u8]) -> Self {
        let mut keys = vec![0; 1042];

        setup_cryptosetup_bb(&mut keys, seed);
        BbCipher {
            keys: keys,
            seed: seed.to_vec()
        }
    }

    /// Get an immutable view of the seed used to generate the keys.
    pub fn get_seed<'a>(&'a self) -> &'a[u8] {
        &self.seed
    }
}

impl Encryptor for BbCipher {
    fn encrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String> {
        use std::num::Wrapping as W;

        let mut ci = Cursor::new(input);
        let mut co = Cursor::new(output);

        // Operate on 2 u32 at a time
        loop {
            let mut n1: W<u32>;
            let n2: W<u32>;
            let mut tmp1: W<u32>;
            let mut tmp2: W<u32>;
            let tmp3: W<u32>;

            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                n1 = W(n)
            } else { return Ok(()) }
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                n2 = W(n)
            } else { return Err("Buffer length not multiple of 8.".to_string()) }

            n1 = n1 ^ W(self.keys[0]);
            tmp1 = (((W(self.keys[(n1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((n1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((n1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((n1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp1 = tmp1 ^ W(self.keys[1]);
            tmp1 = tmp1 ^ n2;

            tmp2 = (((W(self.keys[(tmp1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((tmp1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((tmp1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((tmp1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp2 = tmp2 ^ W(self.keys[2]);
            n1 = n1 ^ tmp2;
            tmp3 = (((W(self.keys[(n1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((n1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((n1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((n1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp1 = tmp1 ^ tmp3 ^ W(self.keys[3]);
            tmp2 = (((W(self.keys[(tmp1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((tmp1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((tmp1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((tmp1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp2 = tmp2 ^ W(self.keys[4]);
            tmp1 = tmp1 ^ W(self.keys[5]);
            n1 = n1 ^ tmp2;

            // Write phase
            if let Err(_) = co.write_u32::<LittleEndian>(tmp1.0) {
                return Ok(())
            }
            if let Err(_) = co.write_u32::<LittleEndian>(n1.0) {
                return Err("Output buffer not multiple of 8.".to_string())
            }
        }
    }
}

impl Decryptor for BbCipher {
    fn decrypt(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String> {
        use std::num::Wrapping as W;

        let mut ci = Cursor::new(input);
        let mut co = Cursor::new(output);

        // Operate on 2 u32 at a time
        loop {
            let mut n1: W<u32>;
            let n2: W<u32>;
            let mut tmp1: W<u32>;
            let mut tmp2: W<u32>;
            let tmp3: W<u32>;

            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                n1 = W(n)
            } else { return Ok(()) }
            if let Ok(n) = ci.read_u32::<LittleEndian>() {
                n2 = W(n)
            } else { return Err("Invalid length".to_string()) }

            n1 = n1 ^ W(self.keys[5]);
            tmp1 = (((W(self.keys[(n1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((n1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((n1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((n1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp1 = tmp1 ^ W(self.keys[4]);
            tmp1 = tmp1 ^ n2;

            tmp2 = (((W(self.keys[(tmp1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((tmp1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((tmp1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((tmp1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp2 = tmp2 ^ W(self.keys[3]);
            n1 = n1 ^ tmp2;
            tmp3 = (((W(self.keys[(n1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((n1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((n1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((n1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp1 = tmp1 ^ tmp3 ^ W(self.keys[2]);
            tmp2 = (((W(self.keys[(tmp1 >> 0x18).0 as usize]) + W(0x12)) + W(self.keys[(((tmp1 >> 0x10) & W(0xFF)) + W(0x112)).0 as usize]))
                    ^ W(self.keys[(((tmp1 >> 0x8) & W(0xFF)) + W(0x212)).0 as usize])) + W(self.keys[((tmp1 & W(0xFF)) + W(0x312)).0 as usize]);
            tmp2 = tmp2 ^ W(self.keys[1]);
            tmp1 = tmp1 ^ W(self.keys[0]);
            n1 = n1 ^ tmp2;

            // Write phase
            if let Err(_) = co.write_u32::<LittleEndian>(tmp1.0) {
                return Ok(())
            }
            if let Err(_) = co.write_u32::<LittleEndian>(n1.0) {
                return Err("Output buffer is not big enough".to_string())
            }
        }
    }
}

/// Generates the key vector for Blue Burst encryption.
/// Taken from Sylverant. Holy hell, I'm pretty sure this is just verbatim disassembly.
pub fn setup_cryptosetup_bb(keys: &mut [u32], salt: &[u8]) {
    use std::num::Wrapping;
    use std::num::Wrapping as W;
    let mut eax: Wrapping<u32>;
    let mut ecx: Wrapping<u32>;
    let mut edx: Wrapping<u32>;
    let mut ebx: Wrapping<u32>;
    let mut ebp: Wrapping<u32>;
    let mut esi: Wrapping<u32>;
    let mut edi: Wrapping<u32>;
    let mut ou: Wrapping<u32>;

    let mut s = [W(0u8); 48];

    if salt.len() != 48 {
        panic!("The seed needs to be 48 bytes long: got {}", salt.len());
    }
    for (i, x) in salt.iter().enumerate() {
        s[i] = W(*x)
    }

    {
        let mut i = 0;
        while i < 48 {
            s[i] = s[i] ^ W(0x19u8);
            s[i + 1] = s[i + 1] ^ W(0x16u8);
            s[i + 2] = s[i + 2] ^ W(0x18u8);
            i += 3;
        }
    }

    keys[0] = 0x243F6A88;
    keys[1] = 0x85A308D3;
    keys[2] = 0x13198A2E;
    keys[3] = 0x03707344;
    keys[4] = 0xA4093822;
    keys[5] = 0x299F31D0;
    keys[6] = 0x082EFA98;
    keys[7] = 0xEC4E6C89;
    keys[8] = 0x452821E6;
    keys[9] = 0x38D01377;
    keys[10] = 0xBE5466CF;
    keys[11] = 0x34E90C6C;
    keys[12] = 0xC0AC29B7;
    keys[13] = 0xC97C50DD;
    keys[14] = 0x3F84D5B5;
    keys[15] = 0xB5470917;
    keys[16] = 0x9216D5D9;
    keys[17] = 0x8979FB1B;

    for (i, x) in BB_TABLE[..].iter().enumerate() {
        keys[18+i] = *x as u32;
    }

    ecx = Wrapping(0);
    ebx = Wrapping(0);

    while ebx < W(0x12) {
        ebp = W(s[ecx.0 as usize].0 as u32) << 0x18;
        eax = ecx + W(1);
        edx = eax - ((eax / W(48))*W(48));
        eax = (W(s[edx.0 as usize].0 as u32) << 0x10) & W(0xFF0000);
        ebp = (ebp | eax) & W(0xffff00ff);
        eax = ecx + W(2);
        edx = eax - ((eax / W(48))*W(48));
        eax = (W(s[edx.0 as usize].0 as u32) << 0x8) & W(0xFF00);
        ebp = (ebp | eax) & W(0xffffff00);
        eax = ecx + W(3);
        ecx = ecx + W(4);
        edx = eax - ((eax / W(48))*W(48));
        eax = W(s[edx.0 as usize].0 as u32);
        ebp = ebp | eax;
        eax = ecx;
        edx = eax - ((eax / W(48))*W(48));
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
        ebx = (esi >> 0x10) & W(0xff);
        eax = W(keys[eax.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
        ebx = (esi >> 8) & W(0xFF);
        eax = eax ^ W(keys[ebx.0 as usize+0x212]);
        ebx = esi & W(0xff);
        eax = eax + W(keys[ebx.0 as usize+0x312]);

        eax = eax ^ W(keys[1]);
        ecx = ecx ^ eax;
        ebx = ecx >> 0x18;
        eax = (ecx >> 0x10) & W(0xFF);
        ebx = W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
        eax = (ecx >> 8) & W(0xff);
        ebx = ebx ^ W(keys[eax.0 as usize+0x212]);
        eax = ecx & W(0xff);
        ebx = ebx + W(keys[eax.0 as usize+0x312]);

        for x in 0..6 {
            ebx = ebx ^ W(keys[(x*2)+2]);
            esi = esi ^ ebx;
            ebx = esi >> 0x18;
            eax = (esi >> 0x10) & W(0xFF);
            ebx = W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
            eax = (esi >> 8) & W(0xff);
            ebx = ebx ^ W(keys[eax.0 as usize+0x212]);
            eax=esi & W(0xff);
            ebx=ebx + W(keys[eax.0 as usize+0x312]);

            ebx=ebx ^ W(keys[(x*2)+3]);
            ecx= ecx ^ ebx;
            ebx=ecx >> 0x18;
            eax=(ecx >> 0x10) & W(0xFF);
            ebx=W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
            eax=(ecx >> 8) & W(0xff);
            ebx=ebx ^ W(keys[eax.0 as usize+0x212]);
            eax=ecx & W(0xff);
            ebx=ebx + W(keys[eax.0 as usize+0x312]);
        }

        ebx=ebx ^ W(keys[14]);
        esi= esi ^ ebx;
        eax=esi >> 0x18;
        ebx=(esi >> 0x10) & W(0xFF);
        eax=W(keys[eax.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
        ebx=(esi >> 8) & W(0xff);
        eax=eax ^ W(keys[ebx.0 as usize+0x212]);
        ebx=esi & W(0xff);
        eax=eax + W(keys[ebx.0 as usize+0x312]);

        eax=eax ^ W(keys[15]);
        eax= ecx ^ eax;
        ecx=eax >> 0x18;
        ebx=(eax >> 0x10) & W(0xFF);
        ecx=W(keys[ecx.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
        ebx=(eax >> 8) & W(0xff);
        ecx=ecx ^ W(keys[ebx.0 as usize+0x212]);
        ebx=eax & W(0xff);
        ecx=ecx + W(keys[ebx.0 as usize+0x312]);

        ecx=ecx ^ W(keys[16]);
        ecx=ecx ^ esi;
        esi= W(keys[17]);
        esi=esi ^ eax;
        keys[(edi.0 as usize / 4)]=esi.0;
        keys[(edi.0 as usize / 4)+1]=ecx.0;
        edi=edi+W(8);
    }

    //eax=W(0);
    //edx=W(0);
    ou=W(0);
    while ou < W(0x1000) {
        edi=W(0x48);
        edx=W(0x448);

        while edi < edx {
            esi=esi ^ W(keys[0]);
            eax=esi >> 0x18;
            ebx=(esi >> 0x10) & W(0xff);
            eax=W(keys[eax.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
            ebx=(esi >> 8) & W(0xFF);
            eax=eax ^ W(keys[ebx.0 as usize+0x212]);
            ebx=esi & W(0xff);
            eax=eax + W(keys[ebx.0 as usize+0x312]);

            eax=eax ^ W(keys[1]);
            ecx= ecx ^ eax;
            ebx=ecx >> 0x18;
            eax=(ecx >> 0x10) & W(0xFF);
            ebx=W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
            eax=(ecx >> 8) & W(0xff);
            ebx=ebx ^ W(keys[eax.0 as usize+0x212]);
            eax=ecx & W(0xff);
            ebx=ebx + W(keys[eax.0 as usize+0x312]);

            for x in 0..6 {
                ebx=ebx ^ W(keys[(x*2)+2]);
                esi= esi ^ ebx;
                ebx=esi >> 0x18;
                eax=(esi >> 0x10) & W(0xFF);
                ebx=W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
                eax=(esi >> 8) & W(0xff);
                ebx=ebx ^ W(keys[eax.0 as usize+0x212]);
                eax=esi & W(0xff);
                ebx=ebx + W(keys[eax.0 as usize+0x312]);

                ebx=ebx ^ W(keys[(x*2)+3]);
                ecx= ecx ^ ebx;
                ebx=ecx >> 0x18;
                eax=(ecx >> 0x10) & W(0xFF);
                ebx=W(keys[ebx.0 as usize+0x12]) + W(keys[eax.0 as usize+0x112]);
                eax=(ecx >> 8) & W(0xff);
                ebx=ebx ^ W(keys[eax.0 as usize+0x212]);
                eax=ecx & W(0xff);
                ebx=ebx + W(keys[eax.0 as usize+0x312]);
            }

            ebx=ebx ^ W(keys[14]);
            esi= esi ^ ebx;
            eax=esi >> 0x18;
            ebx=(esi >> 0x10) & W(0xFF);
            eax=W(keys[eax.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
            ebx=(esi >> 8) & W(0xff);
            eax=eax ^ W(keys[ebx.0 as usize+0x212]);
            ebx=esi & W(0xff);
            eax=eax + W(keys[ebx.0 as usize+0x312]);

            eax=eax ^ W(keys[15]);
            eax= ecx ^ eax;
            ecx=eax >> 0x18;
            ebx=(eax >> 0x10) & W(0xFF);
            ecx=W(keys[ecx.0 as usize+0x12]) + W(keys[ebx.0 as usize+0x112]);
            ebx=(eax >> 8) & W(0xff);
            ecx=ecx ^ W(keys[ebx.0 as usize+0x212]);
            ebx=eax & W(0xff);
            ecx=ecx + W(keys[ebx.0 as usize+0x312]);

            ecx=ecx ^ W(keys[16]);
            ecx=ecx ^ esi;
            esi= W(keys[17]);
            esi=esi ^ eax;
            keys[(ou.0 as usize / 4)+(edi.0 as usize / 4)]=esi.0;
            keys[(ou.0 as usize / 4)+(edi.0 as usize / 4)+1]=ecx.0;
            edi=edi+W(8);
        }
        ou=ou+W(0x400);
    }
}

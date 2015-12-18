//use std::io;

/// Generate a 48-byte random seed for cipher setup.
pub fn gen_seed() -> Vec<u8> {
    use rand::random;
    let mut i = 0;
    let mut ret = vec![0; 48];
    while i < 48 {
        let c: i32 = random();
        ret[i + 0] = (c >> 0) as u8;
        ret[i + 1] = (c >> 8) as u8;
        ret[i + 2] = (c >> 16) as u8;
        ret[i + 3] = (c >> 24) as u8;
        i += 4;
    }
    ret
}

pub mod nsc;

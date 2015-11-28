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

// /// Write a String to a u8 buffer with padding bytes.
// ///
// /// The buffer will be completely cleared of data.
// pub fn string_to_u8<S: Into<String>>(value: S, dst: &mut [u8]) -> io::Result<()> {
//     for x in dst.iter_mut() {
//         *x = 0
//     }
//     let s: String = value.into();
//     for c in s.iter() {
//
//     }
//
//     Ok(())
// }

extern crate byteorder;
extern crate encoding;

/// PC crypto. Used by patch server on BB as well. The Dreamcast and
/// PC versions share this crypto format.
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

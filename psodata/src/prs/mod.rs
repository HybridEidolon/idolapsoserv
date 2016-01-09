//! PRS compression and decompression library.

pub mod compress;
pub mod decompress;

pub use self::compress::compress;
pub use self::decompress::decompress;

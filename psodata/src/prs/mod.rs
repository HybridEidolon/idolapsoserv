//! PRS compression and decompression library.

pub mod compress;
pub mod decompress;

pub use self::compress::compress as compress_prs;
pub use self::decompress::decompress as decompress_prs;

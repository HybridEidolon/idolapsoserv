//! GameCube archive and compression format.

//pub mod compress;
pub mod decompress;

#[derive(Clone, Debug)]
pub struct GslHeader {
    pub name: String,
    pub offset: u32,
    pub size: u32
}

#[derive(Clone, Debug)]
pub struct GslFile {
    pub name: String,
    pub data: Vec<u8>
}

pub use self::decompress::decompress_le;
pub use self::decompress::decompress_be;
pub use self::decompress::decompress_guess;

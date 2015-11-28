extern crate rand;
extern crate crypto;
extern crate psocrypto;
extern crate byteorder;
extern crate encoding;

/// Common message-related types.
pub mod message;

/// Patch server code.
pub mod patch;

/// Game information structures.
pub mod game {
    /// An enumeration of PSO versions.
    pub enum Version {
        /// Phantasy Star Online: Blue Burst
        BlueBurst,
        /// Phantasy Star Online Episodes I & II
        Gamecube,
        /// Phantasy Star Online Episode III: C.A.R.D. Revolution
        Episode3,
        /// Phantasy Star Online PC
        PC,
        /// Phantasy Star Online ver. 2
        DCV2,
        /// Phantasy Star Online
        DCV1
    }
}

/// Utility functions
pub mod util;

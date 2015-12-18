use std::fmt;

/// An enumeration of PSO versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CharClass {
    HUmar,
    HUnewearl,
    HUcast,
    RAmar,
    RAcast,
    RAcaseal,
    FOmarl,
    FOnewm,
    FOnewearl,
    HUcaseal,
    FOmar,
    RAmarl
}
impl fmt::Display for CharClass {
    fn fmt(&self, w: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(w, "{:?}", self)
    }
}

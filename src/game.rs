use std::fmt;

use std::str::FromStr;

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

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Version, Self::Err> {
        use self::Version::*;
        match s {
            "BlueBurst" => Ok(BlueBurst),
            "Gamecube" => Ok(Gamecube),
            "Episode3" => Ok(Episode3),
            "PC" => Ok(PC),
            "DCV2" => Ok(DCV2),
            "DCV1" => Ok(DCV1),
            _ => Err(format!("Unknown version {}", s))
        }
    }
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

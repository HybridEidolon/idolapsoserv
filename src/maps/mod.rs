//! Structures for managing free exploration map data.

use std::fs::File;
use std::io;

use psodata::map::{MapEnemy, MapObject};
use psodata::map::{read_map_enemies, read_map_objects};

pub mod ep1;
pub mod ep2;
pub mod ep4;

pub use self::ep1::Ep1Areas;
pub use self::ep2::Ep2Areas;
pub use self::ep4::Ep4Areas;

/// Enumeration of in-game area IDs for Episode 1.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Ep1AreaCode {
    City = 0,
    Forest1,
    Forest2,
    Cave1,
    Cave2,
    Cave3,
    Mine1,
    Mine2,
    Ruins1,
    Ruins2,
    Ruins3,
    Dragon,
    DeRolLe,
    VolOpt,
    DarkFalz,
    Lobby, // ???
    Temple, // used in battle mode, object data only exists in quests
    Spaceship // used in battle mode
}

/// Enumeration of in-game area IDs for Episode 2.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Ep2AreaCode {
    City = 0,
    TempleAlpha,
    TempleBeta,
    SpaceshipAlpha,
    SpaceshipBeta,
    CentralControlArea,
    JungleNorth,
    JungleEast,
    Mountain,
    Seaside,
    SeabedUpper,
    SeabedLower,
    GalGryphon,
    OlgaFlow,
    BarbaRay,
    GolDragon,
    SeasideNight, // used in a quest I believe?
    Tower // Not any particular floor. The quest controls that.
}

/// Enumeration of in-game area IDs for Episode 4.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Ep4AreaCode {
    City = 0,
    CraterEast,
    CraterWest,
    CraterSouth,
    CraterNorth,
    CraterInterior,
    Desert1,
    Desert2,
    Desert3,
    SaintMilion // also Shambertin (Point of Disaster), Kondrieu (rare),
    // or preferably, the snake asshole that is too hard to hit
}

/// Collection of all areas
#[derive(Clone, Debug)]
pub struct Areas {
    pub ep1: Ep1Areas,
    pub ep2: Ep2Areas,
    pub ep4: Ep4Areas
}

impl Areas {
    /// Load online mode maps
    pub fn load_from_files(path: &str) -> io::Result<Areas> {
        let ep1 = try!(Ep1Areas::load_from_files(path));
        let ep2 = try!(Ep2Areas::load_from_files(path));
        let ep4 = try!(Ep4Areas::load_from_files(path));
        Ok(Areas {
            ep1: ep1,
            ep2: ep2,
            ep4: ep4
        })
    }

    /// Load offline mode maps
    pub fn load_from_files_offline(path: &str) -> io::Result<Areas> {
        let ep1 = try!(Ep1Areas::load_from_files_offline(path));
        let ep2 = try!(Ep2Areas::load_from_files_offline(path));
        let ep4 = try!(Ep4Areas::load_from_files_offline(path));
        Ok(Areas {
            ep1: ep1,
            ep2: ep2,
            ep4: ep4
        })
    }
}

/// An enemy in a lobby instance.
#[derive(Clone, Debug)]
pub struct InstanceEnemy {
    pub param_entry: usize,
    pub rt_entry: usize,
    pub name: &'static str
    // pub clients_hit: u8,
    // pub last_client: u8,
    // pub drop_done: bool
}

/// An object in a lobby instance.
#[derive(Clone, Debug)]
pub struct InstanceObject {
    pub data: MapObject
}

/// Map variation data.
#[derive(Clone, Debug)]
pub struct VariationData {
    pub enemies: Vec<MapEnemy>,
    pub objects: Vec<MapObject>
}

impl VariationData {
    /// Make a variation from two map_.dat files.
    pub fn load_from_files(root: &str, enemies: &str, objects: &str) -> io::Result<VariationData> {
        let mut file = try!(File::open(format!("{}/{}", root, enemies)));
        let e = try!(read_map_enemies(&mut file));
        let mut file = try!(File::open(format!("{}/{}", root, objects)));
        let o = try!(read_map_objects(&mut file));
        Ok(VariationData {
            enemies: e,
            objects: o,
        })
    }
}

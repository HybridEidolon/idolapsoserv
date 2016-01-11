//! Functions for reading from Blue Burst map files in the param directory.

// nine areas per episode (8 in ep4)

use psodata::map::{MapEnemy, MapObject};

pub static EP1_CITY_ONLINE_MAP: &'static (&'static str, &'static str) = &("map_city00_00e.dat", "map_city00_00o.dat");

pub static EP1_FOREST_1_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[
    ("map_forest01_00e.dat", "map_forest01_00o.dat"),
    ("map_forest01_01e.dat", "map_forest01_01o.dat"),
    ("map_forest01_02e.dat", "map_forest01_02o.dat"),
    ("map_forest01_03e.dat", "map_forest01_03o.dat"),
    ("map_forest01_04e.dat", "map_forest01_04o.dat")
];

pub static EP1_FOREST_2_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[
    ("map_forest02_00e.dat", "map_forest02_00o.dat"),
    ("map_forest02_01e.dat", "map_forest02_01o.dat"),
    ("map_forest02_02e.dat", "map_forest02_02o.dat"),
    ("map_forest02_03e.dat", "map_forest02_03o.dat"),
    ("map_forest02_04e.dat", "map_forest02_04o.dat")
];

pub static EP1_CAVES_1_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[
    ("map_cave01_00_00e.dat", "map_cave01_00_00o.dat"),
	("map_cave01_00_01e.dat", "map_cave01_00_01o.dat"),
	("map_cave01_01_00e.dat", "map_cave01_01_00o.dat"),
	("map_cave01_01_01e.dat", "map_cave01_01_01o.dat"),
	("map_cave01_02_00e.dat", "map_cave01_02_00o.dat"),
	("map_cave01_02_01e.dat", "map_cave01_02_01o.dat")
];

pub static EP1_CAVES_2_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

pub static EP1_CAVES_3_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

pub static EP1_MINES_1_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

pub static EP1_MINES_2_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

pub static EP1_RUINS_1_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

pub static EP1_RUINS_2_ONLINE_MAPS: &'static [(&'static str, &'static str)] = &[

];

/// An enemy in a lobby instance.
pub struct InstanceEnemy {
    pub param_entry: usize,
    // pub rt_index: u8,
    // pub clients_hit: u8,
    // pub last_client: u8,
    // pub drop_done: bool
}

/// An object in a lobby instance.
pub struct InstanceObject {
    pub data: MapObject
}

/// Loaded map data.
pub struct MapData {
    pub enemies: Vec<MapEnemy>,
    pub objects: Vec<MapObject>
}

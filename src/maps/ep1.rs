//! Episode 1 maps.

use std::io;
use std::collections::HashMap;

use super::VariationData;


/// Container of all areas' maps for Episode 1.
pub struct Ep1Areas {
    city: VariationData,
    forest1: Vec<VariationData>,
    forest2: Vec<VariationData>,
    cave1: HashMap<(u32, u32), VariationData>,
    cave2: HashMap<(u32, u32), VariationData>,
    cave3: HashMap<(u32, u32), VariationData>,
    machine1: HashMap<(u32, u32), VariationData>,
    machine2: HashMap<(u32, u32), VariationData>,
    ancient1: HashMap<(u32, u32), VariationData>,
    ancient2: HashMap<(u32, u32), VariationData>,
    ancient3: HashMap<(u32, u32), VariationData>,
    boss1: VariationData,
    boss2: VariationData,
    boss3: VariationData,
    boss4: VariationData,
}

impl Ep1Areas {
    /// Load all episode 1 map files.
    pub fn load_from_files(path: &str) -> io::Result<Ep1Areas> {
        unimplemented!()
    }
}

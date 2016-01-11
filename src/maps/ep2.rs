use std::io;
use std::collections::HashMap;

use super::VariationData;

/// Container of all areas' maps for Episode 2.
pub struct Ep2Areas {
    city: VariationData,
    ruins1: HashMap<(u32, u32), VariationData>, // temple, not to be confused with ep1's "ancient"
    ruins2: HashMap<(u32, u32), VariationData>,
    space1: HashMap<(u32, u32), VariationData>,
    space2: HashMap<(u32, u32), VariationData>,
    jungle1: HashMap<(u32, u32), VariationData>,
    jungle2: HashMap<(u32, u32), VariationData>,
    jungle3: HashMap<(u32, u32), VariationData>,
    jungle4: HashMap<(u32, u32), VariationData>,
    jungle5: HashMap<(u32, u32), VariationData>,
    seabed1: HashMap<(u32, u32), VariationData>,
    seabed2: HashMap<(u32, u32), VariationData>,
    boss5: VariationData,
    boss6: VariationData,
    boss7: VariationData,
    boss8: VariationData
}

impl Ep2Areas {
    pub fn load_from_files(path: &str) -> io::Result<Ep2Areas> {
        unimplemented!()
    }
}

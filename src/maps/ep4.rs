use std::collections::HashMap;

use super::VariationData;

/// Container of all areas' maps for Episode 4.
pub struct Ep4Areas {
    city: VariationData,
    wilds: HashMap<(u32, u32), VariationData>,
    desert: HashMap<(u32, u32), VariationData>,
    boss9: VariationData
}

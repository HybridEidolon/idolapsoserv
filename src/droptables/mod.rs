//! Drop tables using the ItemPT and ItemRT structures in `psodata`.

use psodata::itempt::ItemPT;

pub struct DropTable {
    ptables: Vec<ItemPT>
}


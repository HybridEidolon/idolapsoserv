//! Default inventory generator for new characters.

use psodata::chara::*;
use psodata::leveltable::LevelTable;

/// Generate the default inventory for the character given. Also grants default
/// techniques for FOs.
pub fn make_defaults(chara: &mut BbFullCharData, level_table: &LevelTable) {
    let mut ret: Inventory = Default::default();

    match chara.chara.class {
        0 | 1 | 2 | 9 => {
            // HUmar, HUnewearl, HUcast, HUcaseal
            // Saber
            let mut item: InvItem = Default::default();
            item.data.data[1] = 0x01;
            item.exists = 1;
            item.flags = 8; // equipped
            item.data.item_id = 0x00010000;
            ret.items.push(item);
        },
        3 | 4 | 5 | 11 => {
            // RAmar, RAcast, RAcaseal, RAmarl
            // Handgun
            let mut item: InvItem = Default::default();
            item.data.data[1] = 0x06;
            item.exists = 1;
            item.flags = 8; // equipped
            item.data.item_id = 0x00010000;
            ret.items.push(item);
        },
        6 | 7 | 8 | 10 => {
            // FOmarl, FOnewm, FOnewearl, FOmar
            // Cane
            let mut item: InvItem = Default::default();
            item.data.data[1] = 0x0A;
            item.exists = 1;
            item.flags = 8; // equipped
            item.data.item_id = 0x00010000;
            ret.items.push(item);
        },
        e => {
            warn!("Unknown character class {}! No default weapon", e);
        }
    }

    // Frame
    let mut item: InvItem = Default::default();
    item.data.data[0] = 0x01;
    item.data.data[1] = 0x01;
    item.exists = 1;
    item.flags = 8; // equipped
    item.data.item_id = 0x00010001;
    ret.items.push(item);

    // Mag
    let mut item: InvItem = Default::default();
    item.data.data[0] = 0x02;
    item.data.data[2] = 0x05;
    item.data.data[4] = 0xF4;
    item.data.data[5] = 0x01;
    item.data.data2[0] = 0x14; // 20 Synchro by default
    item.exists = 1;
    item.flags = 8; // equipped
    item.data.item_id = 0x00010002;
    // mag color
    match chara.chara.class {
        2 | 9 | 4 | 5 => {
            // CASTs use their skin color (paint job...?)
            item.data.data2[3] = chara.chara.skin as u8;
        },
        _ => {
            // Everyone else is based on costume
            item.data.data2[3] = chara.chara.costume as u8;
        }
    }
    ret.items.push(item);

    // Monomate
    let mut item: InvItem = Default::default();
    item.data.data[0] = 0x03; // Item class
    item.data.data[5] = 0x04; // Stack size
    item.exists = 1;
    item.data.item_id = 0x00010003;
    ret.items.push(item);

    // Monofluid
    match chara.chara.class {
        6 | 7 | 8 | 10 => {
            // Any of the Forces
            let mut item: InvItem = Default::default();
            item.data.data[0] = 0x03; // Item class
            item.data.data[1] = 0x01;
            item.data.data[5] = 0x04; // Stack size
            item.exists = 1;
            item.data.item_id = 0x00010004;
            ret.items.push(item);

            // Bonus: they also get Foie
            chara.chara.techniques[0] = 0x00; // 0xFF indicates unlearned

            // and they have a different default palette
            chara.chara.config = vec![0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0,
                0, 3, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
                0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
                0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
                0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
                0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        },
        _ => ()
    }

    chara.chara.meseta = 300;

    // Default stats
    if let Some(ss) = level_table.start_stats.get(chara.chara.class as usize) {
        chara.chara.stats.atp = ss.atp;
        chara.chara.stats.evp = ss.evp;
        chara.chara.stats.hp = ss.hp;
        chara.chara.stats.mst = ss.mst;
        chara.chara.stats.dfp = ss.dfp;
        chara.chara.stats.lck = ss.lck;
        chara.chara.stats.ata = ss.ata;
    } else {
        // We've already warned about unknown class. Let's give them some HP
        // so they aren't totally bugged, I guess.
        chara.chara.stats.hp = 10;
    }

    // Set character's inventory.
    chara.inv = ret;
}

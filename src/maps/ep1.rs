//! Episode 1 maps.

use std::io;
use std::collections::HashMap;

use super::VariationData;

/// Container of all areas' maps for Episode 1.
#[derive(Clone, Debug)]
pub struct Ep1Areas {
    pub city: VariationData,
    pub forest1: Vec<VariationData>,
    pub forest2: Vec<VariationData>,
    pub cave1: HashMap<(u32, u32), VariationData>,
    pub cave2: HashMap<(u32, u32), VariationData>,
    pub cave3: HashMap<(u32, u32), VariationData>,
    pub machine1: HashMap<(u32, u32), VariationData>,
    pub machine2: HashMap<(u32, u32), VariationData>,
    pub ancient1: HashMap<(u32, u32), VariationData>,
    pub ancient2: HashMap<(u32, u32), VariationData>,
    pub ancient3: HashMap<(u32, u32), VariationData>,
    pub boss1: VariationData, // dragon
    pub boss2: VariationData, // de rol le
    pub boss3: VariationData, // vol opt
    pub boss4: VariationData, // dark falz
}

impl Ep1Areas {
    /// Load all episode 1 map files.
    pub fn load_from_files(path: &str) -> io::Result<Ep1Areas> {
        let city = try!(VariationData::load_from_files(path,
            "map_city00_00e.dat", "map_city00_00o.dat"));
        let forest1 = {
            let mut vs = Vec::with_capacity(5);
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest01_00e.dat", "map_forest01_00o.dat")));
                vs.push(try!(VariationData::load_from_files(path,
                "map_forest01_01e.dat", "map_forest01_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest01_02e.dat", "map_forest01_02o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest01_03e.dat", "map_forest01_03o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest01_04e.dat", "map_forest01_04o.dat")));
            // FUN FACT: VARIANT 6 EXISTS BUT HAS NO PLAYER SPAWNS OR DOORS LOL
            // vs.push(try!(VariationData::load_from_files(path,
            //     "map_forest01_05e.dat", "map_forest01_05o.dat")));
            vs
        };
        let forest2 = {
            let mut vs = Vec::with_capacity(5);
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest02_00e.dat", "map_forest02_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest02_01e.dat", "map_forest02_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest02_02e.dat", "map_forest02_02o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest02_03e.dat", "map_forest02_03o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_forest02_04e.dat", "map_forest02_04o.dat")));
            vs
        };
        let cave1 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_cave01_00_00e.dat", "map_cave01_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_cave01_00_01e.dat", "map_cave01_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_cave01_01_00e.dat", "map_cave01_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_cave01_01_01e.dat", "map_cave01_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_cave01_02_00e.dat", "map_cave01_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_cave01_02_01e.dat", "map_cave01_02_01o.dat")));
            vs
        };
        let cave2 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_cave02_00_00e.dat", "map_cave02_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_cave02_00_01e.dat", "map_cave02_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_cave02_01_00e.dat", "map_cave02_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_cave02_01_01e.dat", "map_cave02_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_cave02_02_00e.dat", "map_cave02_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_cave02_02_01e.dat", "map_cave02_02_01o.dat")));
            vs
        };
        let cave3 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_cave03_00_00e.dat", "map_cave03_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_cave03_00_01e.dat", "map_cave03_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_cave03_01_00e.dat", "map_cave03_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_cave03_01_01e.dat", "map_cave03_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_cave03_02_00e.dat", "map_cave03_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_cave03_02_01e.dat", "map_cave03_02_01o.dat")));
            vs
        };
        let machine1 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_machine01_00_00e.dat", "map_machine01_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_machine01_00_01e.dat", "map_machine01_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_machine01_01_00e.dat", "map_machine01_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_machine01_01_01e.dat", "map_machine01_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_machine01_02_00e.dat", "map_machine01_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_machine01_02_01e.dat", "map_machine01_02_01o.dat")));
            vs
        };
        let machine2 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_machine02_00_00e.dat", "map_machine02_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_machine02_00_01e.dat", "map_machine02_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_machine02_01_00e.dat", "map_machine02_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_machine02_01_01e.dat", "map_machine02_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_machine02_02_00e.dat", "map_machine02_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_machine02_02_01e.dat", "map_machine02_02_01o.dat")));
            vs
        };
        let ancient1 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_ancient01_00_00e.dat", "map_ancient01_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_ancient01_00_01e.dat", "map_ancient01_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_ancient01_01_00e.dat", "map_ancient01_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_ancient01_01_01e.dat", "map_ancient01_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_ancient01_02_00e.dat", "map_ancient01_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_ancient01_02_01e.dat", "map_ancient01_02_01o.dat")));
            vs
        };
        let ancient2 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_ancient02_00_00e.dat", "map_ancient02_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_ancient02_00_01e.dat", "map_ancient02_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_ancient02_01_00e.dat", "map_ancient02_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_ancient02_01_01e.dat", "map_ancient02_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_ancient02_02_00e.dat", "map_ancient02_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_ancient02_02_01e.dat", "map_ancient02_02_01o.dat")));
            vs
        };
        let ancient3 = {
            let mut vs = HashMap::with_capacity(6);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_ancient03_00_00e.dat", "map_ancient03_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_ancient03_00_01e.dat", "map_ancient03_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_ancient03_01_00e.dat", "map_ancient03_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_ancient03_01_01e.dat", "map_ancient03_01_01o.dat")));
            vs.insert((2, 0), try!(VariationData::load_from_files(path,
                "map_ancient03_02_00e.dat", "map_ancient03_02_00o.dat")));
            vs.insert((2, 1), try!(VariationData::load_from_files(path,
                "map_ancient03_02_01e.dat", "map_ancient03_02_01o.dat")));
            vs
        };
        // some of these actually have an ultimate mode variation, but only
        // for objects, so it's not a big deal.
        let boss1 = try!(VariationData::load_from_files(path,
            "map_boss01e.dat", "map_boss01e.dat"));
        let boss2 = try!(VariationData::load_from_files(path,
            "map_boss02e.dat", "map_boss02e.dat"));
        let boss3 = try!(VariationData::load_from_files(path,
            "map_boss03e.dat", "map_boss03e.dat"));
        let boss4 = try!(VariationData::load_from_files(path,
            "map_boss04e.dat", "map_boss04e.dat"));

        Ok(Ep1Areas {
            city: city,
            forest1: forest1,
            forest2: forest2,
            cave1: cave1,
            cave2: cave2,
            cave3: cave3,
            machine1: machine1,
            machine2: machine2,
            ancient1: ancient1,
            ancient2: ancient2,
            ancient3: ancient3,
            boss1: boss1,
            boss2: boss2,
            boss3: boss3,
            boss4: boss4
        })
    }

    /// Load the offline mode variation tables for Episode 1.
    pub fn load_from_files_offline(_path: &str) -> io::Result<Ep1Areas> {
        unimplemented!()
    }
}

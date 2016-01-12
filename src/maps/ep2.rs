use std::io;
use std::collections::HashMap;

use super::VariationData;

/// Container of all areas' maps for Episode 2.
///
/// Fun fact, VR Temple and Spaceship are direct throwbacks to dungeons from
/// classic Phantasy Star, in particular Phantasy Star 4.
#[derive(Clone, Debug)]
pub struct Ep2Areas {
    pub city: VariationData,
    pub ruins1: HashMap<(u32, u32), VariationData>, // temple, not to be confused with ep1's "ancient"
    pub ruins2: HashMap<(u32, u32), VariationData>,
    pub space1: HashMap<(u32, u32), VariationData>,
    pub space2: HashMap<(u32, u32), VariationData>,
    pub jungle1: Vec<VariationData>,
    pub jungle2: Vec<VariationData>,
    pub jungle3: Vec<VariationData>,
    pub jungle4: HashMap<(u32, u32), VariationData>,
    pub jungle5: Vec<VariationData>,
    pub seabed1: HashMap<(u32, u32), VariationData>,
    pub seabed2: HashMap<(u32, u32), VariationData>,
    pub boss5: VariationData, // gal gryphon
    pub boss6: VariationData, // olga flow
    pub boss7: VariationData, // barba ray
    pub boss8: VariationData // gol dragon
}

impl Ep2Areas {
    pub fn load_from_files(path: &str) -> io::Result<Ep2Areas> {
        // files with _s are seasonal variant but we don't actually care
        let city = try!(VariationData::load_from_files(path,
            "map_labo00_00e.dat", "map_labo00_00o.dat"));

        let ruins1 = {
            let mut vs = HashMap::with_capacity(2);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_ruins01_00_00e.dat", "map_ruins01_00_00e.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_ruins01_01_00e.dat", "map_ruins01_01_00e.dat")));
            vs
        };
        let ruins2 = {
            let mut vs = HashMap::with_capacity(2);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_ruins02_00_00e.dat", "map_ruins02_00_00e.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_ruins02_01_00e.dat", "map_ruins02_01_00e.dat")));
            vs
        };
        let space1 = {
            let mut vs = HashMap::with_capacity(2);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_space01_00_00e.dat", "map_space01_00_00e.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_space01_01_00e.dat", "map_space01_01_00e.dat")));
            vs
        };
        let space2 = {
            let mut vs = HashMap::with_capacity(2);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_space02_00_00e.dat", "map_space02_00_00e.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_space02_01_00e.dat", "map_space02_01_00e.dat")));
            vs
        };
        let jungle1 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle01_00e.dat", "map_jungle01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle01_01e.dat", "map_jungle01_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle01_02e.dat", "map_jungle01_02o.dat")));
            vs
        };
        let jungle2 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle02_00e.dat", "map_jungle02_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle02_01e.dat", "map_jungle02_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle02_02e.dat", "map_jungle02_02o.dat")));
            vs
        };
        let jungle3 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle03_00e.dat", "map_jungle03_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle03_01e.dat", "map_jungle03_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle03_02e.dat", "map_jungle03_02o.dat")));
            vs
        };
        let jungle4 = {
            let mut vs = HashMap::with_capacity(4);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_jungle04_00_00e.dat", "map_jungle04_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_jungle04_00_01e.dat", "map_jungle04_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_jungle04_01_00e.dat", "map_jungle04_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_jungle04_01_01e.dat", "map_jungle04_01_01o.dat")));
            vs
        };
        let jungle5 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle05_00e.dat", "map_jungle05_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle05_01e.dat", "map_jungle05_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_jungle05_02e.dat", "map_jungle05_02o.dat")));
            vs
        };
        let seabed1 = {
            let mut vs = HashMap::with_capacity(4);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_seabed01_00_00e.dat", "map_seabed01_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_seabed01_00_01e.dat", "map_seabed01_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_seabed01_01_00e.dat", "map_seabed01_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_seabed01_01_01e.dat", "map_seabed01_01_01o.dat")));
            vs
        };
        let seabed2 = {
            let mut vs = HashMap::with_capacity(4);
            vs.insert((0, 0), try!(VariationData::load_from_files(path,
                "map_seabed02_00_00e.dat", "map_seabed02_00_00o.dat")));
            vs.insert((0, 1), try!(VariationData::load_from_files(path,
                "map_seabed02_00_01e.dat", "map_seabed02_00_01o.dat")));
            vs.insert((1, 0), try!(VariationData::load_from_files(path,
                "map_seabed02_01_00e.dat", "map_seabed02_01_00o.dat")));
            vs.insert((1, 1), try!(VariationData::load_from_files(path,
                "map_seabed02_01_01e.dat", "map_seabed02_01_01o.dat")));
            vs
        };
        let boss5 = try!(VariationData::load_from_files(path,
            "map_boss05e.dat", "map_boss05o.dat"));
        let boss6 = try!(VariationData::load_from_files(path,
            "map_boss06e.dat", "map_boss06o.dat"));
        let boss7 = try!(VariationData::load_from_files(path,
            "map_boss07e.dat", "map_boss07o.dat"));
        let boss8 = try!(VariationData::load_from_files(path,
            "map_boss08e.dat", "map_boss08o.dat"));

        Ok(Ep2Areas {
            city: city,
            ruins1: ruins1,
            ruins2: ruins2,
            space1: space1,
            space2: space2,
            jungle1: jungle1,
            jungle2: jungle2,
            jungle3: jungle3,
            jungle4: jungle4,
            jungle5: jungle5,
            seabed1: seabed1,
            seabed2: seabed2,
            boss5: boss5,
            boss6: boss6,
            boss7: boss7,
            boss8: boss8
        })
    }

    pub fn load_from_files_offline(_path: &str) -> io::Result<Ep2Areas> {
        unimplemented!()
    }
}

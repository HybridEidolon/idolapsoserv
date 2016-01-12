use std::io;

use super::VariationData;

/// Container of all areas' maps for Episode 4.
#[derive(Clone, Debug)]
pub struct Ep4Areas {
    pub city: VariationData,
    pub wilds1: Vec<VariationData>, // east
    pub wilds2: Vec<VariationData>, // west
    pub wilds3: Vec<VariationData>, // south
    pub wilds4: Vec<VariationData>, // north
    pub crater: Vec<VariationData>,
    pub desert1: Vec<VariationData>,
    pub desert2: Vec<VariationData>,
    pub desert3: Vec<VariationData>,
    pub boss9: VariationData // snake asshole
}

impl Ep4Areas {
    /// Load all episode 4 map files.
    pub fn load_from_files(path: &str) -> io::Result<Ep4Areas> {
        let city = try!(VariationData::load_from_files(path,
            "map_city02_00_00e.dat", "map_city02_00_00o.dat"));

        let wilds1 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_00e.dat", "map_wilds01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_01e.dat", "map_wilds01_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_02e.dat", "map_wilds01_00_02o.dat")));
            vs
        };
        let wilds2 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_00e.dat", "map_wilds01_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_01e.dat", "map_wilds01_01_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_02e.dat", "map_wilds01_01_02o.dat")));
            vs
        };
        let wilds3 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_00e.dat", "map_wilds01_02_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_01e.dat", "map_wilds01_02_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_02e.dat", "map_wilds01_02_02o.dat")));
            vs
        };
        let wilds4 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_00e.dat", "map_wilds01_03_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_01e.dat", "map_wilds01_03_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_02e.dat", "map_wilds01_03_02o.dat")));
            vs
        };
        let crater = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_00e.dat", "map_crater01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_01e.dat", "map_crater01_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_02e.dat", "map_crater01_00_02o.dat")));
            vs
        };
        let desert1 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_00_00e.dat", "map_desert01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_01_00e.dat", "map_desert01_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_02_00e.dat", "map_desert01_02_00o.dat")));
            vs
        };
        let desert2 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_00e.dat", "map_desert02_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_01e.dat", "map_desert02_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_02e.dat", "map_desert02_00_02o.dat")));
            vs
        };
        let desert3 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_00_00e.dat", "map_desert03_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_01_00e.dat", "map_desert03_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_02_00e.dat", "map_desert03_02_00o.dat")));
            vs
        };

        let boss9 = try!(VariationData::load_from_files(path,
            "map_boss09_00_00e.dat", "map_boss09_00_00o.dat"));

        Ok(Ep4Areas {
            city: city,
            wilds1: wilds1,
            wilds2: wilds2,
            wilds3: wilds3,
            wilds4: wilds4,
            crater: crater,
            desert1: desert1,
            desert2: desert2,
            desert3: desert3,
            boss9: boss9
        })
    }

    pub fn load_from_files_offline(path: &str) -> io::Result<Ep4Areas> {
        let city = try!(VariationData::load_from_files(path,
            "map_city02_00_00e_s.dat", "map_city02_00_00o_s.dat"));

        let wilds1 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_00e.dat", "map_wilds01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_01e.dat", "map_wilds01_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_00_02e.dat", "map_wilds01_00_02o.dat")));
            vs
        };
        let wilds2 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_00e.dat", "map_wilds01_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_01e.dat", "map_wilds01_01_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_01_02e.dat", "map_wilds01_01_02o.dat")));
            vs
        };
        let wilds3 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_00e.dat", "map_wilds01_02_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_01e.dat", "map_wilds01_02_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_02_02e.dat", "map_wilds01_02_02o.dat")));
            vs
        };
        let wilds4 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_00e.dat", "map_wilds01_03_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_01e.dat", "map_wilds01_03_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_wilds01_03_02e.dat", "map_wilds01_03_02o.dat")));
            vs
        };
        let crater = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_00e.dat", "map_crater01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_01e.dat", "map_crater01_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_crater01_00_02e.dat", "map_crater01_00_02o.dat")));
            vs
        };
        let desert1 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_00_00e.dat", "map_desert01_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_01_00e.dat", "map_desert01_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert01_02_00e.dat", "map_desert01_02_00o.dat")));
            vs
        };
        let desert2 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_00e.dat", "map_desert02_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_01e.dat", "map_desert02_00_01o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert02_00_02e.dat", "map_desert02_00_02o.dat")));
            vs
        };
        let desert3 = {
            let mut vs = Vec::with_capacity(3);
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_00_00e.dat", "map_desert03_00_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_01_00e.dat", "map_desert03_01_00o.dat")));
            vs.push(try!(VariationData::load_from_files(path,
                "map_desert03_02_00e.dat", "map_desert03_02_00o.dat")));
            vs
        };

        let boss9 = try!(VariationData::load_from_files(path,
            "map_boss09_00_00e.dat", "map_boss09_00_00o.dat"));

        Ok(Ep4Areas {
            city: city,
            wilds1: wilds1,
            wilds2: wilds2,
            wilds3: wilds3,
            wilds4: wilds4,
            crater: crater,
            desert1: desert1,
            desert2: desert2,
            desert3: desert3,
            boss9: boss9
        })
    }
}

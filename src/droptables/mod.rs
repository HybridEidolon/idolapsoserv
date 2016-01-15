//! Drop tables using the ItemPT and ItemRT structures in `psodata`.

use std::io::{Cursor, Write, Read};
use std::io;
use std::fs::File;
use std::collections::HashMap;

use psoserial::Serial;

use psodata::itempt::ItemPT;
use psodata::gsl::GslFile;
use psodata::gsl;

/// A struct storing all the probability tables inside an ItemPT.gsl file.
/// Provides methods for retrieving references to them by episode and mode.
pub struct DropTable {
    ep1: Vec<ItemPT>,
    ep2: Vec<ItemPT>,
    ep4: Vec<ItemPT>,
    ep1c: Vec<ItemPT>,
    ep2c: Vec<ItemPT>
}

impl DropTable {
    /// Load a DropTable from an ItemPT.gsl. The file is expected to have the
    /// Episode 4 probability tables appended to the end, even if they don't
    /// exist in the header.
    pub fn load_from_file(path: &str) -> io::Result<DropTable> {
        let mut gsl_file: File = try!(File::open(path));
        let files: Vec<GslFile> = try!(gsl::decompress_guess(&mut gsl_file));
        // Make a map of the files and their data.
        let files = convert_gslfile_vec_to_hash_map(files);

        // Each episode has 4 difficulties to consider
        // challenge 1 and 2 are considered separate "episodes" here
        // Build the Episode 1 list
        let ep1 = {
            
        };
        unimplemented!()
    }
}

fn convert_gslfile_vec_to_hash_map(files: Vec<GslFile>) -> HashMap<String, Vec<u8>> {
    let mut ret = HashMap::new();
    for f in files {
        ret.insert(f.name, f.data);
    }
    ret
}

fn build_ep1_vec(files: &HashMap<String, Vec<u8>>) -> Result<Vec<ItemPT>, String> {
    let mut ret = Vec::with_capacity(4);
    let normal_filenames: Vec<_> = (0..10).into_iter().map(|i| format!("ItemPTn{}.ref", i)).collect();
    let hard_filenames: Vec<_> = (0..10).into_iter().map(|i| format!("ItemPTh{}.ref", i)).collect();
    let vhard_filenames: Vec<_> = (0..10).into_iter().map(|i| format!("ItemPTv{}.ref", i)).collect();
    let ultimate_filenames: Vec<_> = (0..10).into_iter().map(|i| format!("ItemPTu{}.ref", i)).collect();
    
    let normal_files = {
        let mut r = Vec::new();
        for f in normal_filenames.iter() {

        }
    };
    unimplemented!()
}

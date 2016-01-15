//! Drop tables using the ItemPT and ItemRT structures in `psodata`.

use std::io;
use std::fs::File;
use std::collections::HashMap;

use psodata::itempt::ItemPT;
use psodata::itemrt::ItemRT;
use psodata::gsl::GslFile;
use psodata::gsl;

/// A struct storing all the probability tables inside an ItemPT.gsl file.
/// Provides methods for retrieving references to them by episode and mode.
pub struct DropTable {
    ep1: Option<Vec<ItemPT>>,
    ep2: Option<Vec<ItemPT>>,
    ep4: Option<Vec<ItemPT>>,
    ep1c: Option<Vec<ItemPT>>,
    ep2c: Option<Vec<ItemPT>>,

    rt_ep1: Option<Vec<ItemRT>>,
    rt_ep2: Option<Vec<ItemRT>>,
    rt_ep4: Option<Vec<ItemRT>>
}

impl DropTable {
    /// Load a DropTable from an ItemPT.gsl. The file is expected to have the
    /// Episode 4 probability tables appended to the end, even if they don't
    /// exist in the header.
    pub fn load_from_file(item_pt: &str, item_rt: &str) -> io::Result<DropTable> {
        let mut gsl_file: File = try!(File::open(item_pt));
        let files: Vec<GslFile> = try!(gsl::decompress_guess(&mut gsl_file));
        // Make a map of the files and their data.
        let files = convert_gslfile_vec_to_hash_map(files);

        for i in files.keys() {
            debug!("ItemPT.gsl file: {}", i);
        }

        // Each episode has 4 difficulties to consider
        // challenge 1 and 2 are considered separate "episodes" here
        let ep1 = build_pt_ep_vec(&files, "").map_err(|e| warn!("ItemPT.gsl error: {:?}", e)).ok();
        let ep2 = build_pt_ep_vec(&files, "l").map_err(|e| warn!("ItemPT.gsl error: {:?}", e)).ok();
        let ep4 = build_pt_ep_vec(&files, "bb").map_err(|e| warn!("ItemPT.gsl error: {:?}", e)).ok();
        let ep1c = build_pt_ep_vec(&files, "c").map_err(|e| warn!("ItemPT.gsl error: {:?}", e)).ok();
        let ep2c = build_pt_ep_vec(&files, "cl").map_err(|e| warn!("ItemPT.gsl error: {:?}", e)).ok();

        let mut gsl_file: File = try!(File::open(item_rt));
        let files: Vec<GslFile> = try!(gsl::decompress_guess(&mut gsl_file));
        // Make a map of the files and their data.
        let files = convert_gslfile_vec_to_hash_map(files);

        for i in files.keys() {
            debug!("ItemRT.gsl file: {}", i);
        }

        let rt_ep1 = build_rt_ep_vec(&files, "").map_err(|e| warn!("ItemRT.gsl error: {:?}", e)).ok();
        let rt_ep2 = build_rt_ep_vec(&files, "l").map_err(|e| warn!("ItemRT.gsl error: {:?}", e)).ok();
        let rt_ep4 = build_rt_ep_vec(&files, "bb").map_err(|e| warn!("ItemRT.gsl error: {:?}", e)).ok();
        // does challenge mode not use rare tables?
        //let rt_ep1c = build_rt_ep_vec(&files, "c").map_err(|e| warn!("ItemRT.gsl error: {:?}", e)).ok();

        if ep1.is_none() {
            warn!("Failed to load episode 1 probability tables from {}; drops will not be available for this episode", item_pt);
        }
        if ep2.is_none() {
            warn!("Failed to load episode 2 probability tables from {}; drops will not be available for this episode", item_pt);
        }
        if ep4.is_none() {
            warn!("Failed to load episode 4 probability tables from {}; drops will not be available for this episode", item_pt);
        }
        if ep1c.is_none() {
            warn!("Failed to load episode 1 challenge probability tables from {}; drops will not be available for this episode", item_pt);
        }
        if ep2c.is_none() {
            warn!("Failed to load episode 2 challenge probability tables from {}; drops will not be available for this episode", item_pt);
        }

        if rt_ep1.is_none() {
            warn!("Failed to load episode 1 rare tables from {}; rare drops will not be available for this episode", item_rt);
        }

        if rt_ep2.is_none() {
            warn!("Failed to load episode 2 rare tables from {}; rare drops will not be available for this episode", item_rt);
        }

        if rt_ep4.is_none() {
            warn!("Failed to load episode 4 rare tables from {}; rare drops will not be available for this episode", item_rt);
        }

        Ok(DropTable {
            ep1: ep1,
            ep2: ep2,
            ep4: ep4,
            ep1c: ep1c,
            ep2c: ep2c,
            rt_ep1: rt_ep1,
            rt_ep2: rt_ep2,
            rt_ep4: rt_ep4
        })
    }
}

fn convert_gslfile_vec_to_hash_map(files: Vec<GslFile>) -> HashMap<String, Vec<u8>> {
    let mut ret = HashMap::new();
    for f in files {
        ret.insert(f.name, f.data);
    }
    ret
}

fn build_pt_ep_vec(files: &HashMap<String, Vec<u8>>, episode: &str) -> io::Result<Vec<ItemPT>> {
    let mut ret = Vec::with_capacity(4);
    let normal_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemPT{}n{}.rel", episode, i)).collect();
    let hard_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemPT{}h{}.rel", episode, i)).collect();
    let vhard_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemPT{}v{}.rel", episode, i)).collect();
    let ultimate_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemPT{}u{}.rel", episode, i)).collect();

    let mut r = Vec::new();
    for f in normal_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemPT.gsl", f)))
        }
    }
    ret.push(try!(ItemPT::load_from_buffers(&r[..])));
    let mut r = Vec::new();
    for f in hard_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemPT.gsl", f)))
        }
    }
    ret.push(try!(ItemPT::load_from_buffers(&r[..])));let mut r = Vec::new();
    for f in vhard_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemPT.gsl", f)))
        }
    }
    ret.push(try!(ItemPT::load_from_buffers(&r[..])));let mut r = Vec::new();
    for f in ultimate_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemPT.gsl", f)))
        }
    }
    ret.push(try!(ItemPT::load_from_buffers(&r[..])));
    Ok(ret)
}

fn build_rt_ep_vec(files: &HashMap<String, Vec<u8>>, episode: &str) -> io::Result<Vec<ItemRT>> {
    let mut ret = Vec::with_capacity(4);
    let normal_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemRT{}n{}.rel", episode, i)).collect();
    let hard_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemRT{}h{}.rel", episode, i)).collect();
    let vhard_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemRT{}v{}.rel", episode, i)).collect();
    let ultimate_filenames: Vec<_> = (0..10)
        .into_iter().map(|i| format!("ItemRT{}u{}.rel", episode, i)).collect();

    let mut r = Vec::new();
    for f in normal_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemRT.gsl", f)))
        }
    }
    ret.push(try!(ItemRT::load_from_buffers(&r[..])));
    let mut r = Vec::new();
    for f in hard_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemRT.gsl", f)))
        }
    }
    ret.push(try!(ItemRT::load_from_buffers(&r[..])));let mut r = Vec::new();
    for f in vhard_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemRT.gsl", f)))
        }
    }
    ret.push(try!(ItemRT::load_from_buffers(&r[..])));let mut r = Vec::new();
    for f in ultimate_filenames.iter() {
        match files.get(f) {
            Some(buf) => r.push(&buf[..]),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!("missing file {} in ItemRT.gsl", f)))
        }
    }
    ret.push(try!(ItemRT::load_from_buffers(&r[..])));
    Ok(ret)
}

use std::io;
use std::io::Read;
use std::fs::File;

use psomsg::bb::*;

use crc::crc32::checksum_ieee as crc32;

pub fn load_paramfiles_msgs(data_root: &str) -> io::Result<(Message, Vec<Message>)> {
    let paramfiles = [
        "ItemMagEdit.prs",
        "ItemPMT.prs",
        "BattleParamEntry.dat",
        "BattleParamEntry_on.dat",
        "BattleParamEntry_lab.dat",
        "BattleParamEntry_lab_on.dat",
        "BattleParamEntry_ep4.dat",
        "BattleParamEntry_ep4_on.dat",
        "PlyLevelTbl.prs"
    ];

    let mut param_file_data: Vec<io::Result<(String, Vec<u8>, u32)>> = paramfiles.iter().map(|filename| {
        let mut f = try!(File::open(format!("{}/param/{}", data_root, filename)));
        let mut buf = Vec::new();
        try!(f.read_to_end(&mut buf));
        let checksum = crc32(&buf[..]);
        Ok((filename.to_string(), buf, checksum))
    }).collect();

    let mut param_headers = Vec::new();
    let mut param_buffer = Vec::new();

    let mut offset = 0;
    for r in param_file_data.iter_mut() {
        match r {
            &mut Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Couldn't load params")),
            &mut Ok((ref filen, ref mut buf, ref checksum)) => {
                param_headers.push(ParamHeader {
                    size: buf.len() as u32,
                    checksum: *checksum,
                    offset: offset,
                    filename: filen.clone()
                });
                param_buffer.append(buf);
                offset += buf.len() as u32;
            }
        }
    }

    let header_msg = Message::BbParamHdr(paramfiles.len() as u32, BbParamHdr {
        params: param_headers
    });

    let mut buffers = Vec::new();
    // Split the total buffer into several 0x6800 messages
    while param_buffer.len() > 0x6800 {
        let remaining = param_buffer.split_off(0x6800);
        buffers.push(param_buffer);
        param_buffer = remaining;
    }
    buffers.push(param_buffer);

    let param_chunks = buffers.into_iter().enumerate().map(|(i, v)| {
        Message::BbParamChunk(0, BbParamChunk {
            chunk: i as u32,
            data: v
        })
    }).collect();
    Ok((header_msg, param_chunks))
}

extern crate idola;
extern crate psocrypto;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate encoding;

use std::net::*;
use std::io::{Read, Write, BufReader, BufWriter, Cursor};
use std::io;
use std::fs::{File, Metadata};
use std::thread;

use idola::message::bb::*;

use psocrypto::*;

fn handle_stream(mut stream: TcpStream, key_table: Vec<u32>) {
    let peer_addr = stream.peer_addr().unwrap();

    info!("Blue Burst client {} connected", peer_addr);

    // make new ciphers
    let client_key = vec![0u8; 48];
    let server_key = vec![0u8; 48];
    let client_cipher = BbCipher::new(&client_key, &key_table);
    let server_cipher = BbCipher::new(&server_key, &key_table);

    let welcome = Message::Welcome(0, Welcome(server_key, client_key));
    info!("Welcomed BB client {}", peer_addr);

    welcome.serialize(&mut stream).unwrap();

    // now, wrap the stream with encrypt/decrypt
    //let mut w_s = EncryptWriter::new(stream.try_clone().unwrap(), server_cipher);
    let mut r_s = DecryptReader::new(stream.try_clone().unwrap(), client_cipher);

    // REMOVE THIS
    // {
    //     let mut buf = vec![0u8; 8];
    //     stream.read(&mut buf);
    //     debug!("{:?}", buf);
    //     return
    // }

    loop {
        let m = Message::deserialize(&mut r_s).unwrap();
        match m {
            Message::Unknown(o, f, b) => {
                info!("type {}, flags {}, {:?}", o, f, b);
            },
            a => warn!("Received an unexpected but known message: {:?}", a)
        }
    }

}

fn read_key_table(mut f: File) -> io::Result<Vec<u32>> {
    let metadata = try!(f.metadata());
    if metadata.len() != 1042 * 4 {
        return Err(io::Error::new(io::ErrorKind::Other, "key table is not correct size"))
    }

    drop(metadata);
    let mut data = Vec::with_capacity(1042 * 4);
    try!(f.read_to_end(&mut data));
    let mut key_table: Vec<u32> = Vec::with_capacity(1042);
    let mut cur = Cursor::new(data);
    loop {
        use byteorder::{LittleEndian, ReadBytesExt};
        match cur.read_u32::<LittleEndian>() {
            Err(_) => break,
            Ok(n) => key_table.push(n)
        }
    }
    Ok(key_table)
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init().unwrap();

    // Read 1042 BB key table from data/crypto

    let key_table = read_key_table(File::open("data/crypto/bb_table.bin").unwrap()).unwrap();
    info!("Read Blue Burst encryption key table from data/crypto/bb_table.bin");
    debug!("The first few values are {:x}, {:x}, {:x}, {:x}", key_table[0], key_table[1], key_table[2], key_table[3]);

    BbCipher::new(&vec![0u8; 48], &key_table);

    let tcp_listener = TcpListener::bind("0.0.0.0:12000").unwrap();
    for s in tcp_listener.incoming() {
        match s {
            Ok(s) => {
                let kt_clone = key_table.clone();
                thread::spawn(move|| handle_stream(s, kt_clone));
            },
            Err(e) => error!("error, quitting: {}", e)
        }
    }
}

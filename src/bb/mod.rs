use psomsg::bb::*;

use std::net::{TcpStream};
use std::sync::Arc;
use std::io;
use std::io::{Read, Cursor};

use rand::random;

use psocrypto::{DecryptReader, EncryptWriter, BbCipher};

pub struct Context {
    stream: TcpStream,
    key_table: Arc<Vec<u32>>
}

impl Context {
    pub fn new(stream: TcpStream, key_table: Arc<Vec<u32>>) -> Context {
        Context {
            stream: stream,
            key_table: key_table
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        let peer_addr = self.stream.peer_addr().unwrap();

        info!("Blue Burst client {} connected", peer_addr);

        // make new ciphers
        let client_key = vec![random::<u8>(); 48];
        let server_key = vec![random::<u8>(); 48];
        let client_cipher = BbCipher::new(&client_key, &self.key_table);
        let server_cipher = BbCipher::new(&server_key, &self.key_table);

        let welcome = Message::Welcome(0, Welcome(server_key, client_key));
        info!("Welcomed BB client {}", peer_addr);

        welcome.serialize(&mut self.stream).unwrap();

        // now, wrap the stream with encrypt/decrypt
        let mut w_s = EncryptWriter::new(self.stream.try_clone().unwrap(), server_cipher);
        let mut r_s = DecryptReader::new(self.stream.try_clone().unwrap(), client_cipher);

        loop {
            let m = Message::deserialize(&mut r_s).unwrap();
            match m {
                Message::Unknown(o, f, b) => {
                    info!("type {}, flags {}, {:?}", o, f, b);
                },
                Message::Login(_, Login { username, .. }) => {
                    info!("[{}] login attempt with username {}! that's cute...", peer_addr, username);
                    let r = Message::BbSecurity(0, BbSecurity {
                        err_code: 7, // PERMABANNED
                        tag: 0,
                        guildcard: 0,
                        team_id: 0,
                        security_data: vec![0u8; 40],
                        caps: 0
                    });
                    r.serialize(&mut w_s).unwrap();
                    return Ok(())
                }
                a => warn!("Received an unexpected but known message: {:?}", a)
            }
        }
    }
}

/// Utility function to read a key table from a Read
pub fn read_key_table(r: &mut Read) -> io::Result<Vec<u32>> {
    let mut data = Vec::with_capacity(1042 * 4);
    try!(r.read_to_end(&mut data));
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

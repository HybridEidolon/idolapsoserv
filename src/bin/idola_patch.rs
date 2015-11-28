extern crate idola;
extern crate crypto;
extern crate psocrypto;
extern crate byteorder;
extern crate rand;

use std::thread;
use std::io;
use std::io::{Write, Read, Cursor};
use std::net::{TcpListener, TcpStream};
use std::error::Error;

use idola::message::{MessageEncode, MessageDecode};
//use idola::message::patch::Message;

use psocrypto::PcCipher;

use crypto::symmetriccipher::{Encryptor, Decryptor, SymmetricCipherError};
use crypto::buffer::{RefReadBuffer, RefWriteBuffer};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use rand::random;

#[derive(Clone, Copy)]
enum ClientState {
    Connected,
    Welcomed
}

struct ClientContext {
    pub stream: TcpStream,
    pub state: ClientState,
    pub client_cipher: PcCipher,
    pub server_cipher: PcCipher
}

impl ClientContext {
    fn read_ack(&mut self, enc: bool) -> io::Result<(u16, u16)> {
        let size;
        let ty;
        let mut rbuf = [0u8; 4];
        let mut ebuf = [0u8; 4];
        let mut cur;
        {try!(self.stream.read(&mut rbuf));}
        if enc {
            {
                if let Err(e) = self.client_cipher.decrypt(
                    &mut RefReadBuffer::new(&rbuf[..]),
                    &mut RefWriteBuffer::new(&mut ebuf[..]), false) {
                    return Err(io::Error::new(io::ErrorKind::Other, format!("unable to encrypt: {:?}", e)))
                }
            }
            {
                cur = Cursor::new(&ebuf);
                size = cur.read_u16::<LittleEndian>().unwrap();
                ty = cur.read_u16::<LittleEndian>().unwrap();
            }
        } else {
            cur = Cursor::new(&rbuf);
            size = cur.read_u16::<LittleEndian>().unwrap();
            ty = cur.read_u16::<LittleEndian>().unwrap();
        }
        Ok((size, ty))
    }

    fn write_ack(&mut self, enc: bool, ack: (u16, u16)) -> io::Result<()> {
        let (size, ty) = ack;
        let mut wbuf = [0u8; 4];
        let mut ebuf = [0u8; 4];
        {
            let mut cur = Cursor::new(&mut wbuf[..]);
            try!(cur.write_u16::<LittleEndian>(size));
            try!(cur.write_u16::<LittleEndian>(ty));
        }
        if enc {
            if let Err(e) = self.server_cipher.encrypt(
                &mut RefReadBuffer::new(& wbuf[..]),
                &mut RefWriteBuffer::new(&mut ebuf), false) {
                Err(io::Error::new(
                    io::ErrorKind::Other, format!("unable to encrypt ack message: {:?}", e)))
            } else {
                try!(self.stream.write_all(&ebuf));
                self.stream.flush()
            }
        } else {
            try!(self.stream.write_all(&wbuf));
            self.stream.flush()
        }
    }
}

unsafe impl Send for ClientState {}
unsafe impl Send for ClientContext {}

fn handle_client(mut ctx: ClientContext) {
    use idola::message::patch::*;

    let w = Welcome {
        server_vector: ctx.server_cipher.seed(),
        client_vector: ctx.client_cipher.seed()
    };
    w.encode_msg(&mut ctx.stream, None).unwrap();
    ctx.stream.flush().unwrap();

    println!("sent a hello!");

    ctx.state = ClientState::Welcomed;



    // Client will send a Welcome message as an ack. We reply with a Login ack.
    {
        let (size, ty) = match ctx.read_ack(true) { Ok(o) => o, _ => return };
        if size == 4 || ty == 2 {
            match ctx.write_ack(true, (4, 4)) { Ok(_) => (), _ => return }
        } else {
            // hey man we ain't playin, knock it off.
            return;
        }
    }

    loop {
        // Read message
        match Message::decode_msg(&mut ctx.stream as &mut Read, Some(&mut ctx.client_cipher as &mut Decryptor)) {
            Ok(m) => match m {
                Message::Login(Login { .. }) => {
                    println!("they logged in!");
                    let motd = Motd {
                        message: "fake".to_string()
                    };
                    match motd.encode_msg(&mut ctx.stream, Some(&mut ctx.server_cipher as &mut Encryptor)) {
                        Err(e) => {println!("couldn't send motd: {}", e); return},
                        _ => ()
                    }
                    ctx.stream.flush().unwrap();
                },
                _ => println!("they did something else!")
            },
            Err(e) => {println!("message parse error: {}", e.description()); return}
        };
    };
}

fn main() {
    let tcp_listener = TcpListener::bind("127.0.0.1:11000").unwrap();
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(s) => {
                let ctx = ClientContext {
                    stream: s,
                    state: ClientState::Connected,
                    client_cipher: PcCipher::new(0),
                    server_cipher: PcCipher::new(0)
                };
                thread::spawn(move|| handle_client(ctx));
            },
            Err(e) => {println!("Error: could not accept connection from {}", e);}
        };
    }
    println!("Placeholders");
}

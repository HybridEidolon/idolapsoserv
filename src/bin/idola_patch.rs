extern crate idola;
extern crate crypto;
extern crate psocrypto;
extern crate byteorder;
extern crate rand;
#[macro_use] extern crate log;
extern crate env_logger;

use std::thread;
use std::io;
use std::io::{Write, Read, Cursor};
use std::net::{TcpListener, TcpStream};
use std::error::Error;

use idola::message::{MessageEncode, MessageDecode};
use idola::message::patch::Message;

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

    /// Send a message struct.
    fn send_msg(&mut self, msg: &MessageEncode, encrypt: bool) -> io::Result<()> {
        if encrypt {
            try!(msg.encode_msg(&mut self.stream as &mut Write, Some(&mut self.server_cipher)));
            self.stream.flush()
        } else {
            try!(msg.encode_msg(&mut self.stream as &mut Write, None));
            self.stream.flush()
        }
    }

    /// Receive a message enum. Will
    fn recv_msg(&mut self, decrypt: bool) -> io::Result<Message> {
        if decrypt {
            let m = Message::decode_msg(&mut self.stream as &mut Read, Some(&mut self.client_cipher as &mut Decryptor));
            match m {
                Ok(m) => {debug!("msg recv: {:?}", m); Ok(m)},
                e => e
            }
        } else {
            let m = Message::decode_msg(&mut self.stream as &mut Read, None);
            match m {
                Ok(m) => {debug!("msg recv (unenc): {:?}", m); Ok(m)},
                e => e
            }
        }
    }
}

unsafe impl Send for ClientState {}
unsafe impl Send for ClientContext {}

fn handle_client(mut ctx: ClientContext) {
    use idola::message::patch::*;

    let peer_addr = ctx.stream.peer_addr().unwrap();

    info!("client {} connected", peer_addr);

    let w = Welcome {
        server_vector: ctx.server_cipher.seed(),
        client_vector: ctx.client_cipher.seed()
    };
    if let Err(e) = ctx.send_msg(&w, false) {
        error!("unable to send welcome message to {}: {}", peer_addr, e);
        return
    }

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
        use std::net::Ipv4Addr;
        use std::str::FromStr;
        // Read message
        if let Ok(s) = ctx.recv_msg(true) {match s {
            Message::Login(Login { .. } ) => {
                let motd = Motd {
                    message: "how am I gonna feed all these little... BABS\nhey there\n\n:)".to_string()
                };
                ctx.send_msg(&motd, true).unwrap();
                let red = Redirect { ip_addr: Ipv4Addr::from_str("127.0.0.1").unwrap(), port: 11001 };
                ctx.send_msg(&red, true).unwrap();
                // Now we break out of this connection.
                return;
            },
            _ => println!("uhhh")
        }}
    };
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init().unwrap();

    info!("IDOLA Phantasy Star Online Patch Server");
    info!("Version 0.1.0");

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

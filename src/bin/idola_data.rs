//! Runs the data provision server, as a component of the patch server.
extern crate rand;
extern crate crypto;
extern crate psocrypto;
extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate byteorder;

use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io;
use std::io::{Cursor, Read, Write};

use crypto::symmetriccipher::{Decryptor, Encryptor};
use crypto::buffer::{RefReadBuffer, RefWriteBuffer};
use psocrypto::PcCipher;

use rand::random;

use idola::message::{MessageEncode, MessageDecode};
use idola::message::patch::*;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

struct ClientContext {
    pub stream: TcpStream,
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

fn handle_client(mut ctx: ClientContext) {
    let peer = ctx.stream.peer_addr().unwrap();

    info!("connected {}", peer);

    let w = Welcome {
        client_vector: ctx.client_cipher.seed(),
        server_vector: ctx.server_cipher.seed()
    };
    ctx.send_msg(&w, false).unwrap();

    if let Ok((4, 2)) = ctx.read_ack(true) {
        ctx.write_ack(true, (4, 4)).unwrap();
    } else { return }

    loop {
        let m = ctx.recv_msg(true);
        if let Ok(m) = m {match m {
            Message::Login(Some(Login { .. })) => {
                ctx.send_msg(&StartList, true).unwrap();
                ctx.send_msg(&SetDirectory { dirname: vec![46] }, true).unwrap();
                ctx.send_msg(&InfoFinished, true).unwrap();
                ctx.send_msg(&FileListDone, true).unwrap();
            },
            Message::FileListDone(Some(_)) => {
                ctx.send_msg(&SetDirectory { dirname: vec![46] }, true).unwrap();
                ctx.send_msg(&OneDirUp, true).unwrap();
                ctx.send_msg(&SendDone, true).unwrap();
                info!("client {} was updated successfully", peer);
            }
            e => info!("recv something else! {:?}", e)
        }} else {
            return
        }
    }

    //info!("disconnecting {}", peer);
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init().unwrap();

    info!("IDOLA Phantasy Star Online Data Server");
    info!("Version 0.1.0");

    let tcp_listener = TcpListener::bind("127.0.0.1:11001").unwrap();
    info!("Started");
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move|| handle_client(ClientContext {
                    stream: s,
                    client_cipher: PcCipher::new(random()),
                    server_cipher: PcCipher::new(random())
                }));
            },
            _ => ()
        };
    };
}

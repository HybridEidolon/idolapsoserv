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
use std::io::{Read, Write};
use std::fmt::Debug;

use crypto::symmetriccipher::{Decryptor};
use psocrypto::PcCipher;

use rand::random;

use idola::message::{MessageEncode, MessageDecode};
use idola::message::patch::*;
use idola::message::staticvec::StaticVec;

struct ClientContext {
    pub stream: TcpStream,
    pub client_cipher: PcCipher,
    pub server_cipher: PcCipher
}

impl ClientContext {
    /// Send a message struct.
    fn send_msg<T: MessageEncode + Debug>(&mut self, msg: &T, encrypt: bool) -> io::Result<()> {
        if encrypt {
            debug!("msg send: {:?}", msg);
            try!(msg.encode_msg(&mut self.stream as &mut Write, Some(&mut self.server_cipher)));
            self.stream.flush()
        } else {
            debug!("msg send (unenc): {:?}", msg);
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

    if let Ok(Message::Welcome(None)) = ctx.recv_msg(true) {
        ctx.send_msg(&Message::Login(None), true).unwrap();
    }

    loop {
        let m = ctx.recv_msg(true);
        if let Ok(m) = m {match m {
            Message::Login(Some(_)) => {
                ctx.send_msg(&StartList, true).unwrap();
                ctx.send_msg(&SetDirectory { dirname: StaticVec::default() }, true).unwrap();
                ctx.send_msg(&InfoFinished, true).unwrap();
                //ctx.send_msg(&FileListDone, true).unwrap();
            },
            Message::FileListDone(_) => {
                ctx.send_msg(&SetDirectory { dirname: StaticVec::default() }, true).unwrap();
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

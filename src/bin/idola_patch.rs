extern crate idola;
extern crate crypto;
extern crate psocrypto;
extern crate byteorder;
extern crate rand;
#[macro_use] extern crate log;
extern crate env_logger;

use std::thread;
use std::io;
use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream};

use idola::message::{MessageEncode, MessageDecode};
use idola::message::patch::Message;

use psocrypto::PcCipher;

use crypto::symmetriccipher::Decryptor;

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

    /// Receive a message enum.
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
    info!("client {} welcomed", peer_addr);

    ctx.state = ClientState::Welcomed;

    // Client will send a Welcome message as an ack. We reply with a Login ack.
    if let Ok(s) = ctx.recv_msg(true) {
        info!("received a message after being welcomed");
        if s == Message::Welcome(None) {
            info!("sending login ack");
            match ctx.send_msg(&Message::Login(None), true) { Err(_) => return, _ => () }
        }
    }

    loop {
        use std::net::Ipv4Addr;
        use std::str::FromStr;
        // Read message
        if let Ok(s) = ctx.recv_msg(true) {match s {
            Message::Login(Some(_)) => {
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

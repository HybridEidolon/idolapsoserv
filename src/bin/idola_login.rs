extern crate idola;
extern crate psocrypto;
#[macro_use] extern crate log;
extern crate env_logger;

use std::net::*;
use std::io::{BufReader, BufWriter};
use std::thread;

use idola::message::bb::*;

use psocrypto::*;

fn handle_stream(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();

    println!("{} connected", peer_addr);

    // make new ciphers
    let client_key = vec![0u8; 48];
    let server_key = vec![0u8; 48];
    let client_cipher = BbCipher::new(&client_key);
    let server_cipher = BbCipher::new(&server_key);

    let welcome = Message::Welcome(0, Welcome(server_key, client_key));

    welcome.serialize(&mut stream).unwrap();

    // now, wrap the stream with encrypt/decrypt
    let mut w_s = BufWriter::with_capacity(8, EncryptWriter::new(stream.try_clone().unwrap(), server_cipher));
    let mut r_s = BufReader::with_capacity(8, DecryptReader::new(stream.try_clone().unwrap(), client_cipher));

    loop {
        let m = Message::deserialize(&mut r_s).unwrap();
        match m {
            Message::Unknown(o, f, b) => {
                println!("type {}, flags {}, {:?}", o, f, b);
            },
            _ => println!("fuck")
        }
    }

}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init().unwrap();
    let tcp_listener = TcpListener::bind("0.0.0.0:12000").unwrap();
    for s in tcp_listener.incoming() {
        match s {
            Ok(s) => {
                thread::spawn(move|| handle_stream(s));
            },
            Err(e) => println!("error, quitting: {}", e)
        }
    }
}

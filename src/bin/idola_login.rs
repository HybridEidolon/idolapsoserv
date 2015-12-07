extern crate idola;

use std::net::*;
use std::thread;

use idola::message::bb::*;

fn handle_stream(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();

    println!("{} connected", peer_addr);

    let welcome = Message::Welcome(0, Welcome(vec![0u8; 48], vec![0u8; 48]));

    welcome.serialize(&mut stream).unwrap();

    loop {
        let m = Message::deserialize(&mut stream).unwrap();
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

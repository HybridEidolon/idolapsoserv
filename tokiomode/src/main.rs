#[feature(conservative_impl_trait)]

#[macro_use] extern crate log;
extern crate env_logger;
extern crate rand;
extern crate byteorder;
extern crate psocrypto;
extern crate psomsg_patch;
extern crate tokio_core;
extern crate futures;
extern crate psoserial;

use std::net::ToSocketAddrs;
use std::net::Shutdown;
use std::io::{Write, Cursor};

use futures::Future;
use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream, TcpListener, Incoming};
use tokio_core::io;
use psomsg_patch::*;
use psoserial::Serial;

fn main() {
    let mut core: Core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "0.0.0.0:11000".to_socket_addrs().unwrap().next().unwrap();

    let listener: TcpListener = TcpListener::bind(&addr, &handle).unwrap();

    let incoming = listener.incoming();

    println!("Waiting for connections on 11000.");

    let write_four_bytes = incoming.and_then(|(stream, addr)| {

        println!("connected {}", addr);

        let msg = Message::Welcome(Some(Welcome { server_vector: 0, client_vector: 0 }));
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor);
        let buf = cursor.into_inner();
        io::write_all(stream, buf)
    });

    let read_four_bytes = write_four_bytes.and_then(|(stream, buf)| {
        println!("bytes written: {:?}", buf);
        let buf = [0u8; 4];
        io::read_exact(stream, buf)
    });

    let disconnect = read_four_bytes.for_each(|(stream, buf)| {
        println!("bytes read: {:?}", buf);
        stream.shutdown(Shutdown::Both)
    });

    core.run(disconnect).unwrap();
}
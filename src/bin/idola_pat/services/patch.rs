//! The patch service, the root which redirects to data services.

use std::collections::HashMap;

use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

use std::thread;

use std::net::SocketAddr;

use mio::tcp::TcpListener;
use mio::Sender;

pub struct PatchService {
    receiver: Receiver<ServiceMsg>,
    _sender: Sender<LoopMsg>
}

impl PatchService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>) -> Service {
        let (tx, rx) = channel();

        let mut listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        thread::spawn(move|| {
            let p = PatchService {
                receiver: rx,
                _sender: sender
            };
            p.run()
        });

        Service::new(listener, tx)
    }

    pub fn run(self) {
        let PatchService {
            receiver,
            ..
        } = self;

        println!("patch service running");

        // This service only responds to events; it does not run its own bookkeeping.
        for msg in receiver.iter() {
            match msg {
                ServiceMsg::ClientConnected(id) => {
                    println!("Client {} connected to patch service", id);
                    // TODO welcome it
                },
                ServiceMsg::ClientDisconnected(id) => {
                    println!("Client {} disconnected from patch service.", id)
                },
                ServiceMsg::ClientSaid(id, m) => {
                    println!("Client {} said a thing: {:?}", id, m)
                }
            }
        }
    }
}

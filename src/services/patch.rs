//! The patch service, the root which redirects to data services.

use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

use std::thread;

use std::net::SocketAddr;

use mio::tcp::TcpListener;
use mio::Sender;

use psomsg::patch::*;

pub struct PatchService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>
}

impl PatchService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        thread::spawn(move|| {
            let p = PatchService {
                receiver: rx,
                sender: sender
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
                    let w = Box::new(Message::Welcome(Some(Welcome { server_vector: 0, client_vector: 0 })));
                    self.sender.send(LoopMsg::Client(id, w)).unwrap();
                },
                ServiceMsg::ClientDisconnected(id) => {
                    println!("Client {} disconnected from patch service.", id)
                },
                ServiceMsg::ClientSaid(id, m) => {
                    match m.as_ref() {
                        &Message::Welcome(None) => {
                            self.sender.send(LoopMsg::Client(id, Box::new(
                                Message::Login(None)
                            ))).unwrap();
                        },
                        &Message::Login(Some(..)) => {
                            self.sender.send(LoopMsg::Client(id, Box::new(
                                Message::Motd(Some(Motd { message: "Hi there\nfriend".to_string() }))
                            ))).unwrap();
                            // TODO send redirect;
                        },
                        _ => {
                            warn!("weird message sent by client");
                        }
                    }
                }
            }
        }
    }
}

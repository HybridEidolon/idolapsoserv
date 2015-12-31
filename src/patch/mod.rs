//! The patch service, the root which redirects to data services.

use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

use std::thread;

use std::net::{SocketAddr, SocketAddrV4};

use mio::tcp::TcpListener;
use mio::Sender;

use psomsg::patch::*;

use ::services::message::NetMsg;

use ::services::ServiceType;

use rand::random;

pub struct PatchService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    v4_servers: Vec<SocketAddrV4>,
    next: usize,
    motd: String,
    random_data: bool
}

impl PatchService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, v4_servers: Vec<SocketAddrV4>, motd: String, random_data: bool) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        if v4_servers.len() == 0 { panic!("no data redirect servers specified") }

        thread::spawn(move|| {
            let p = PatchService {
                receiver: rx,
                sender: sender,
                v4_servers: v4_servers,
                next: 0,
                motd: motd,
                random_data: random_data
            };
            p.run()
        });

        Service::new(listener, tx, ServiceType::Patch)
    }

    pub fn run(mut self) {
        info!("Patch service running");

        // This service only responds to events; it does not run its own bookkeeping.
        for msg in self.receiver.iter() {
            match msg {
                ServiceMsg::ClientConnected(id) => {
                    println!("Client {} connected to patch service", id);
                    let w = Message::Welcome(Some(Welcome { server_vector: 0, client_vector: 0 }));
                    self.sender.send(LoopMsg::Client(id, w.into())).unwrap();
                },
                ServiceMsg::ClientDisconnected(id) => {
                    println!("Client {} disconnected from patch service.", id)
                },
                ServiceMsg::ClientSaid(id, NetMsg::Patch(m)) => {
                    match m {
                        Message::Welcome(None) => {
                            self.sender.send(LoopMsg::Client(id,
                                Message::Login(None).into()
                            )).unwrap();
                        },
                        Message::Login(Some(..)) => {
                            self.sender.send(LoopMsg::Client(id,
                                Message::Motd(Some(Motd { message: self.motd.clone() })).into()
                            )).unwrap();

                            self.sender.send(LoopMsg::Client(id,
                                Message::Redirect(Some(Redirect(self.v4_servers[self.next]))).into()
                            )).unwrap();
                            self.sender.send(LoopMsg::DropClient(id)).unwrap();

                            if self.random_data {
                                self.next = random();
                            } else {
                                self.next += 1;
                            }
                            self.next %= self.v4_servers.len();
                        },
                        _ => {
                            warn!("weird message sent by client");
                        }
                    }
                },
                _ => { unreachable!() }
            }
        }
    }
}

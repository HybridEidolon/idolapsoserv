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

use ::services::message::NetMsg;

use ::services::ServiceType;

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

        Service::new(listener, tx, ServiceType::Patch)
    }

    pub fn run(self) {
        let PatchService {
            receiver,
            sender
        } = self;

        info!("Patch service running");

        // This service only responds to events; it does not run its own bookkeeping.
        for msg in receiver.iter() {
            match msg {
                ServiceMsg::ClientConnected(id) => {
                    println!("Client {} connected to patch service", id);
                    let w = Message::Welcome(Some(Welcome { server_vector: 0, client_vector: 0 }));
                    sender.send(LoopMsg::Client(id, w.into())).unwrap();
                },
                ServiceMsg::ClientDisconnected(id) => {
                    println!("Client {} disconnected from patch service.", id)
                },
                ServiceMsg::ClientSaid(id, NetMsg::Patch(m)) => {
                    match m {
                        Message::Welcome(None) => {
                            sender.send(LoopMsg::Client(id,
                                Message::Login(None).into()
                            )).unwrap();
                        },
                        Message::Login(Some(..)) => {
                            sender.send(LoopMsg::Client(id,
                                Message::Motd(Some(Motd { message: "Hi there\nfriend".to_string() })).into()
                            )).unwrap();
                            // TODO send redirect;
                            sender.send(LoopMsg::Client(id,
                                Message::Redirect(Some(Redirect("127.0.0.1:11001".parse().unwrap()))).into()
                            )).unwrap();
                            sender.send(LoopMsg::DropClient(id)).unwrap();
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

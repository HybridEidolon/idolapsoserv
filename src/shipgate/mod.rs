//! The Shipgate handles all database requests, propagates certain cross-ship
//! communication (simple mail, etc), and provides the ship list to other
//! services. The ServiceMsg enum has responses for messages sent to it.
//! The Shipgate connection does not sit in the event loop; the client
//! connection is a conventional blocking socket handled on a separate thread.

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::net::SocketAddr;
use std::collections::HashMap;

use mio::tcp::TcpListener;
use mio::Sender;

use ::services::message::NetMsg;
use ::services::{ServiceType, Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use ::shipgate::msg::*;

pub mod msg;
pub mod client;

pub struct ShipGateService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    password: String,
    clients: HashMap<usize, ClientCtx>
}

#[derive(Clone)]
struct ClientCtx {
    authenticated: bool
}

impl Default for ClientCtx {
    fn default() -> ClientCtx {
        ClientCtx {
            authenticated: false
        }
    }
}

impl ShipGateService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, password: &str) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let pw = password.to_owned();
        thread::spawn(move|| {
            let p = ShipGateService {
                receiver: rx,
                sender: sender,
                password: pw,
                clients: Default::default()
            };
            p.run()
        });

        Service::new(listener, tx, ServiceType::ShipGate)
    }

    pub fn run(mut self) {
        info!("ShipGate service running");

        for msg in self.receiver.iter() {
            match msg {
                ServiceMsg::ClientConnected(id) => {
                    info!("Client {} connected to shipgate", id);
                    // create a new client context
                    let ctx = ClientCtx::default();
                    self.clients.insert(id, ctx);
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected from shipgate.", id);
                    self.clients.remove(&id);
                },
                ServiceMsg::ClientSaid(id, NetMsg::ShipGate(m)) => {
                    let c = self.clients[&id].clone();
                    if c.authenticated {
                        match m {
                            Message::BbLoginChallenge(res, BbLoginChallenge { client_id, .. }) => {
                                // TODO comm with db
                                self.sender.send((id, Message::BbLoginChallengeAck(res, BbLoginChallengeAck { client_id: client_id, status: 2 })).into()).unwrap()
                            },
                            _ => () // silently ignore
                        }
                    } else {
                        if let Message::Auth(res, Auth(version, pw)) = m {
                            if version == 0 && pw == self.password {
                                self.clients.get_mut(&id).unwrap().authenticated = true;
                                self.sender.send((id, Message::AuthAck(res, AuthAck)).into()).unwrap();
                                info!("Shipgate client {} successfully authenticated", id);
                                continue
                            } else {
                                info!("Shipgate client {} failed to authenticate", id);
                                self.sender.send(LoopMsg::DropClient(id)).unwrap();
                                continue
                            }
                        } else {
                            // Client must auth first. Drop immediately.
                            info!("Shipgate client {} tried to do something other than Auth first", id);
                            self.sender.send(LoopMsg::DropClient(id)).unwrap();
                            continue
                        }
                    }
                },
                _ => unreachable!()
            }
        }
    }
}

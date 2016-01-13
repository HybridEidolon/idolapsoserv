//! The Shipgate handles all database requests, propagates certain cross-ship
//! communication (simple mail, etc), and provides the ship list to other
//! services. The ServiceMsg enum has responses for messages sent to it.
//! The Shipgate connection does not sit in the event loop; the client
//! connection is a conventional blocking socket handled on a separate thread.

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::net::{SocketAddr, SocketAddrV4};
use std::collections::HashMap;
use std::sync::Arc;

use mio::tcp::TcpListener;
use mio::Sender;

use psodb_common::pool::Pool;

use ::services::message::NetMsg;
use ::services::{ServiceType, Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use ::shipgate::msg::*;

pub mod msg;
pub mod client;
mod handler;

use self::handler::MsgHandler;

pub struct ShipGateService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    password: String,
    clients: HashMap<usize, ClientCtx>,
    pool: Arc<Pool>,
    ships: HashMap<usize, (SocketAddrV4, String)>
}


#[derive(Clone)]
pub struct ClientCtx {
    id: usize,
    authenticated: bool,
    addr: SocketAddr
}

impl ShipGateService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, password: &str, pool: Arc<Pool>) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let pw = password.to_owned();
        thread::spawn(move|| {
            let p = ShipGateService {
                receiver: rx,
                sender: sender,
                password: pw,
                clients: Default::default(),
                pool: pool,
                ships: Default::default()
            };
            p.run()
        });

        Service::new(listener, tx, ServiceType::ShipGate)
    }

    pub fn run(mut self) {
        info!("ShipGate service running");

        loop {
            let msg = match self.receiver.recv() {
                Ok(m) => m,
                Err(_) => return // receiver closed; we can exit service
            };
            match msg {
                ServiceMsg::ClientConnected((addr, id)) => {
                    info!("Client {} connected to shipgate", id);
                    // create a new client context
                    let ctx = ClientCtx {
                        id: id,
                        authenticated: false,
                        addr: addr
                    };
                    self.clients.insert(id, ctx);
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected from shipgate.", id);
                    self.clients.remove(&id);
                },
                ServiceMsg::ClientSaid(id, NetMsg::ShipGate(m)) => {
                    let mut c = match self.clients.get_mut(&id) {
                        Some(c) => c,
                        None => unreachable!()
                    };

                    if c.authenticated {
                        let mut handler = MsgHandler::new(self.pool.clone(), c);
                        let response: Option<(u32, Message)> = match m {
                            Message::BbLoginChallenge(req, body) => {
                                Some((req, handler.handle_login_challenge(body).into()))
                            },
                            Message::BbGetAccountInfo(req, body) => {
                                Some((req, handler.handle_get_bb_account_info(body).into()))
                            },
                            Message::RegisterShip(req, body) => {
                                // Register the ship.
                                info!("Ship {} at {:?} registered", body.1, body.0);
                                self.ships.insert(id, (body.0, body.1));
                                Some((req, RegisterShipAck.into()))
                            },
                            Message::ShipList(req, _) => {
                                let ships: Vec<_> = self.ships.values().map(|v| v.clone()).collect();
                                Some((req, ShipListAck(ships).into()))
                            },
                            Message::BbUpdateOptions(_, body) => {
                                handler.handle_bb_update_options(body);
                                None
                            },
                            Message::BbUpdateKeys(_, body) => {
                                handler.handle_bb_update_keys(body);
                                None
                            },
                            Message::BbUpdateJoy(_, body) => {
                                handler.handle_bb_update_joy(body);
                                None
                            }
                            _ => unimplemented!()
                        };
                        if let Some((req, mut response)) = response {
                            debug!("Client Request {} received from client {}", req, id);
                            response.set_response_key(req);
                            self.sender.send((id, response).into()).unwrap();
                        }
                    } else {
                        if let Message::Auth(res, Auth(version, pw)) = m {
                            if version == 0 && pw == self.password {
                                c.authenticated = true;
                                self.sender.send((id, Message::AuthAck(res, AuthAck)).into()).unwrap();
                                info!("Shipgate client {} successfully authenticated", id);
                                continue
                            } else {
                                warn!("Shipgate client {} failed to authenticate", id);
                                self.sender.send(LoopMsg::DropClient(id)).unwrap();
                                continue
                            }
                        } else {
                            // Client must auth first. Drop immediately.
                            warn!("Shipgate client {} tried to do something other than Auth first", id);
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

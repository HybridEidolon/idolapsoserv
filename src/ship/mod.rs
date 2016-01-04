//! Ship service runner.

use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::cell::RefCell;

use mio::tcp::TcpListener;
use mio::Sender;

use rand::random;

use psomsg::bb::*;

use ::services::message::NetMsg;

use ::services::ServiceType;

use ::shipgate::client::SgSender;
use ::shipgate::client::callbacks::SgCbMgr;
use ::shipgate::msg::RegisterShip;

pub mod handler;
pub mod client;

use self::handler::ShipHandler;

use self::client::ClientState;

pub struct ShipService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<ShipHandler>,
    clients: Rc<RefCell<HashMap<usize, ClientState>>>,
    name: String
}

impl ShipService {
    pub fn spawn(bind: &SocketAddr,
                 sender: Sender<LoopMsg>,
                 key_table: Arc<Vec<u32>>,
                 sg_sender: &SgSender,
                 name: &str) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let sg_sender = sg_sender.clone_with(tx.clone());

        let name = name.to_string();

        thread::spawn(move|| {
            let d = ShipService {
                receiver: rx,
                sender: sender,
                sg_sender: sg_sender.into(),
                clients: Default::default(),
                name: name
            };
            d.run();
        });

        // TODO this isn't going to work for accepting connections from any version
        Service::new(listener, tx, ServiceType::Bb(key_table))
    }

    fn make_handler(&mut self, client_id: usize) -> ShipHandler {
        ShipHandler::new(
            self.sender.clone(),
            self.sg_sender.clone(),
            client_id,
            self.clients.clone()
        )
    }

    pub fn run(mut self) {
        info!("Ship service running.");

        self.sg_sender.send(RegisterShip("127.0.0.1:13000".parse().unwrap(), self.name.clone())).unwrap();

        loop {
            let msg = match self.receiver.recv() {
                Ok(m) => m,
                Err(_) => return
            };

            match msg {
                ServiceMsg::ClientConnected(id) => {
                    info!("Client {} connected to ship {}", id, self.name);
                    let sk = vec![random(); 48];
                    let ck = vec![random(); 48];
                    self.sender.send((id, Message::BbWelcome(0, BbWelcome(sk, ck))).into()).unwrap();

                    // Add to clients table
                    let cs = ClientState::default();
                    {self.clients.borrow_mut().insert(id, cs);}
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected from ship {}", id, self.name);
                    {self.clients.borrow_mut().remove(&id);}
                },
                ServiceMsg::ClientSaid(id, NetMsg::Bb(m)) => {
                    let mut h = self.make_handler(id);
                    match m {
                        Message::BbLogin(_, m) => { h.bb_login(m) },
                        a => {
                            info!("{:?}", a)
                        }
                    }
                },
                ServiceMsg::ShipGateMsg(m) => {
                    let req = m.get_response_key();
                    info!("Shipgate Request {}: Response received", req);
                    let cb;
                    {
                        cb = self.sg_sender.cb_for_req(req)
                    }

                    match cb {
                        Some((client, mut c)) => c(self.make_handler(client), m),
                        None => warn!("Got a SG request response for an unexpected request ID {}.", req)
                    }
                }
                _ => unreachable!()
            }
        }
    }
}

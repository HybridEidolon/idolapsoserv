//! Blue Burst's Login server only officially redirects to a separate
//! Character server to handle character information, but this is almost
//! pointless. IDOLA instead handles both the Login and Character steps inside
//! the BB Login server.

use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use mio::tcp::TcpListener;
use mio::Sender;

use psomsg::bb::*;

use rand::random;

use ::services::message::NetMsg;
use ::services::ServiceType;
use ::login::paramfiles::load_paramfiles_msgs;

use ::shipgate::client::SgSender;
use ::shipgate::client::callbacks::SgCbMgr;

pub mod client;
pub mod handler;

use self::client::ClientState;
use self::handler::BbLoginHandler;

pub struct BbLoginService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BbLoginHandler>,
    clients: Rc<RefCell<HashMap<usize, ClientState>>>,
    param_files: Rc<(Message, Vec<Message>)>
}

impl BbLoginService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, key_table: Arc<Vec<u32>>, sg_sender: &SgSender, data_path: &str) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let sg_sender = sg_sender.clone_with(tx.clone());

        // load param data
        let params = load_paramfiles_msgs(data_path).expect("Couldn't load param files from data path");

        thread::spawn(move|| {
            let d = BbLoginService {
                receiver: rx,
                sender: sender,
                sg_sender: sg_sender.into(),
                clients: Default::default(),
                param_files: Rc::new(params)
            };
            d.run()
        });

        Service::new(listener, tx, ServiceType::Bb(key_table))
    }

    fn make_handler(&mut self, client_id: usize) -> BbLoginHandler {
        BbLoginHandler::new(
            self.sender.clone(),
            self.sg_sender.clone(),
            client_id,
            self.clients.clone(),
            self.param_files.clone()
        )
    }

    pub fn run(mut self) {
        info!("Blue burst login service running");

        loop {
            let msg = match self.receiver.recv() {
                Ok(m) => m,
                Err(_) => return
            };

            match msg {
                ServiceMsg::ClientConnected(id) => {
                    info!("Client {} connected", id);
                    let sk = vec![random(); 48];
                    let ck = vec![random(); 48];
                    self.sender.send((id, Message::BbWelcome(0, BbWelcome(sk, ck))).into()).unwrap();

                    {
                        let mut b = self.clients.borrow_mut();
                        b.insert(id, ClientState::default());
                    }
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected", id);

                    {
                        let mut b = self.clients.borrow_mut();
                        b.remove(&id);
                    }
                },
                ServiceMsg::ClientSaid(id, NetMsg::Bb(m)) => {
                    let mut h = self.make_handler(id);
                    match m {
                        Message::BbLogin(_, m) => { h.bb_login(m) },
                        Message::BbOptionRequest(_, _) => { h.bb_option_request() },
                        Message::BbChecksum(_, m) => { h.bb_checksum(m) },
                        Message::BbGuildRequest(_, _) => { h.bb_guildcard_req() },
                        Message::BbGuildCardChunkReq(_, r) => { h.bb_guildcard_chunk_req(r) },
                        Message::BbCharSelect(_, m) => { h.bb_char_select(m) },
                        Message::BbParamHdrReq(_, _) => { h.bb_param_hdr_req() },
                        Message::BbParamChunkReq(c, _) => { h.bb_param_chunk_req(c) },
                        Message::BbCharInfo(_, m) => { h.bb_char_info(m) },
                        Message::MenuSelect(_, m) => { h.menu_select(m) },
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

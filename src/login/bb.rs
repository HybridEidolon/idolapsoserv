use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::net::SocketAddr;

use mio::tcp::TcpListener;
use mio::Sender;

use psomsg::bb::*;

use ::services::message::NetMsg;

use ::services::ServiceType;

use std::sync::Arc;
use rand::random;

use ::shipgate::client::SgSender;
use ::shipgate::client::callbacks::SgCbMgr;
use ::shipgate::msg::{BbLoginChallenge,
    BbLoginChallengeAck,
    Message as Sgm};


pub struct BbLoginService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BbLoginHandler>
}

struct BbLoginHandler {
    sender: Sender<LoopMsg>,
    sg_sender: SgCbMgr<BbLoginHandler>,
    client_id: usize
}

impl BbLoginHandler {
    fn new(sender: Sender<LoopMsg>, sg_sender: SgCbMgr<BbLoginHandler>, client_id: usize) -> BbLoginHandler {
        BbLoginHandler {
            sender: sender,
            sg_sender: sg_sender,
            client_id: client_id
        }
    }

    fn handle_login(&mut self, m: BbLogin) {
        // on this server, we need to contact the shipgate
        // and verify credentials, then forward to any of
        // the ships for the character step.
        info!("Client {} attempted BB Login", self.client_id);
        let m = BbLoginChallenge { username: m.username.clone(), password: m.password.clone() };
        self.sg_sender.request(self.client_id, m, move|mut h, m| h.handle_sg_login_ack(m)).unwrap();
    }

    fn handle_sg_login_ack(&mut self, m: Sgm) {
        info!("Shipgate login ack");
        if let Sgm::BbLoginChallengeAck(_, BbLoginChallengeAck { status, .. }) = m {
            info!("Shipgate acknowledged login request.");
            let mut sdata = BbSecurityData::default();
            sdata.magic = 0xCAFEB00B;
            let r = Message::BbSecurity(0, BbSecurity {
                err_code: status,
                tag: 0x00010000,
                guildcard: 1000000,
                team_id: 1,
                security_data: sdata,
                caps: 0x00000102
            });
            self.sender.send((self.client_id, r).into()).unwrap();
            if status == 0 {
                info!("User logged in successfully, redirecting to a ship.");
                let r = Message::Redirect(0, Redirect {
                    ip: "127.0.0.1".parse().unwrap(),
                    port: 12001
                });
                self.sender.send((self.client_id, r).into()).unwrap();
            }
        }
    }
}

impl BbLoginService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, key_table: Arc<Vec<u32>>, sg_sender: &SgSender) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let sg_sender = sg_sender.clone_with(tx.clone());

        thread::spawn(move|| {
            let d = BbLoginService {
                receiver: rx,
                sender: sender,
                sg_sender: sg_sender.into()
            };
            d.run()
        });

        Service::new(listener, tx, ServiceType::Bb(key_table))
    }

    fn make_handler(&mut self, client_id: usize) -> BbLoginHandler {
        BbLoginHandler::new(
            self.sender.clone(),
            self.sg_sender.clone(),
            client_id
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
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected", id);
                },
                ServiceMsg::ClientSaid(id, NetMsg::Bb(m)) => {
                    let mut h = self.make_handler(id);
                    match m {
                        Message::BbLogin(_, m) => {
                            h.handle_login(m)
                        },
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

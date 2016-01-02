use ::services::{Service, ServiceMsg};
use ::loop_handler::LoopMsg;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender as MpscSender};

use std::thread;

use std::net::SocketAddr;

use mio::tcp::TcpListener;
use mio::Sender;

use psomsg::bb::*;

use ::services::message::NetMsg;

use ::services::ServiceType;

use std::sync::Arc;
use rand::random;

use ::shipgate::client::ShipGateSender;
use ::shipgate::msg::{BbLoginChallenge,
    BbLoginChallengeAck,
    Message as Sgm};

pub struct BbLoginService {
    receiver: Receiver<ServiceMsg>,
    my_sender: MpscSender<ServiceMsg>,
    sender: Sender<LoopMsg>,
    sg_sender: ShipGateSender
}

impl BbLoginService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, key_table: Arc<Vec<u32>>, sg_sender: ShipGateSender) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        let my_sender = tx.clone();

        thread::spawn(move|| {
            let d = BbLoginService {
                receiver: rx,
                sender: sender,
                my_sender: my_sender,
                sg_sender: sg_sender
            };
            d.run()
        });

        Service::new(listener, tx, ServiceType::Bb(key_table))
    }

    pub fn run(mut self) {
        info!("Blue burst login service running");

        for msg in self.receiver.iter() {
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
                    match m {
                        Message::BbLogin(_, BbLogin { username, password, .. }) => {
                            // on this server, we need to contact the shipgate
                            // and verify credentials, then forward to any of
                            // the ships for the character step.
                            self.sg_sender.send(self.my_sender.clone(), BbLoginChallenge {
                                client_id: id as u64,
                                username: username,
                                password: password
                            }.into()).unwrap();
                            // let mut sdata = BbSecurityData::default();
                            // sdata.magic = 0xCAFEB00B;
                            // let r = Message::BbSecurity(0, BbSecurity {
                            //     err_code: 0, // login success
                            //     tag: 0x00010000,
                            //     guildcard: 1000000,
                            //     team_id: 1,
                            //     security_data: sdata,
                            //     caps: 0x00000102
                            // });
                            // self.sender.send((id, r).into()).unwrap();
                            // let r = Message::Redirect(0, Redirect {
                            //     ip: "127.0.0.1".parse().unwrap(),
                            //     port: 12001
                            // });
                            // self.sender.send((id, r).into()).unwrap();
                        },
                        a => {
                            info!("{:?}", a)
                        }
                    }
                },
                ServiceMsg::ShipGateMsg(m) => {
                    match m {
                        Sgm::BbLoginChallengeAck(_, BbLoginChallengeAck { client_id, status }) => {
                            match status {
                                0 => {
                                    // Success
                                    let mut sdata = BbSecurityData::default();
                                    sdata.magic = 0xCAFEB00B;
                                    let r = Message::BbSecurity(0, BbSecurity {
                                        err_code: 0, // login success
                                        tag: 0x00010000,
                                        guildcard: 1000000,
                                        team_id: 1,
                                        security_data: sdata,
                                        caps: 0x00000102
                                    });
                                    self.sender.send((client_id as usize, r).into()).unwrap();

                                    // self.sender.send((id, r).into()).unwrap();
                                    // let r = Message::Redirect(0, Redirect {
                                    //     ip: "127.0.0.1".parse().unwrap(),
                                    //     port: 12001
                                    // });
                                    // self.sender.send((id, r).into()).unwrap();
                                },
                                e => {
                                    let r = Message::BbSecurity(0, BbSecurity {
                                        err_code: e, // bad pw
                                        tag: 0,
                                        guildcard: 0,
                                        team_id: 0,
                                        security_data: BbSecurityData::default(),
                                        caps: 0
                                    });
                                    self.sender.send((client_id as usize, r).into()).unwrap();
                                },
                            }
                        },
                        _ => ()
                    }
                }
                _ => unreachable!()
            }
        }
    }
}

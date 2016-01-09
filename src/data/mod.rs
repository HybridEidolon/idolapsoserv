//! The data service, an extension of the patch service.

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

pub struct DataService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>
}

impl DataService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        thread::spawn(move|| {
            let d = DataService {
                receiver: rx,
                sender: sender
            };
            d.run()
        });

        Service::new(listener, tx, ServiceType::Patch)
    }

    pub fn run(self) {
        let DataService {
            receiver,
            sender
        } = self;

        info!("Data service running");

        for msg in receiver.iter() {
            match msg {
                ServiceMsg::ClientConnected((_addr, id)) => {
                    info!("Client {} connected to data service", id);
                    let w = Message::Welcome(Some(Welcome { server_vector: 0, client_vector: 0 }));
                    sender.send(LoopMsg::Client(id, w.into())).unwrap();
                },
                ServiceMsg::ClientDisconnected(id) => {
                    info!("Client {} disconnected from data service.", id)

                },
                ServiceMsg::ClientSaid(id, NetMsg::Patch(m)) => {
                    match m {
                        Message::Welcome(None) => {
                            sender.send(LoopMsg::Client(id,
                                Message::Login(None).into()
                            )).unwrap();
                        },
                        Message::Login(Some(..)) => {
                            // Message::StartList(None).serialize(&mut w_s).unwrap();
                            // Message::SetDirectory(Some(SetDirectory { dirname: StaticVec::default() })).serialize(&mut w_s).unwrap();
                            // Message::InfoFinished(None).serialize(&mut w_s).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::StartList(None).into()
                            )).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::SetDirectory(Some(SetDirectory { dirname: Default::default() })).into()
                            )).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::InfoFinished(None).into()
                            )).unwrap();
                        },
                        Message::FileListDone(_) => {
                            // Message::SetDirectory(Some(SetDirectory { dirname: StaticVec::default() })).serialize(&mut w_s).unwrap();
                            // Message::OneDirUp(None).serialize(&mut w_s).unwrap();
                            // Message::SendDone(None).serialize(&mut w_s).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::SetDirectory(Some(SetDirectory { dirname: Default::default() })).into()
                            )).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::OneDirUp(None).into()
                            )).unwrap();
                            sender.send(LoopMsg::Client(id,
                                Message::SendDone(None).into()
                            )).unwrap();
                            info!("client {} was 'updated' successfully", id);
                        },
                        u => { warn!("client sent weird message: {:?}", u) }
                    }
                },
                _ => unreachable!()
            }
        }
    }
}

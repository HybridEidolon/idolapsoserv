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

pub struct BbLoginService {
    receiver: Receiver<ServiceMsg>,
    sender: Sender<LoopMsg>
}

impl BbLoginService {
    pub fn spawn(bind: &SocketAddr, sender: Sender<LoopMsg>, key_table: Arc<Vec<u32>>) -> Service {
        let (tx, rx) = channel();

        let listener = TcpListener::bind(bind).expect("Couldn't create tcplistener");

        thread::spawn(move|| {
            let d = BbLoginService {
                receiver: rx,
                sender: sender
            };
            d.run()
        });

        Service::new(listener, tx, ServiceType::Bb(key_table))
    }

    pub fn run(self) {
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
                ServiceMsg::ClientSaid(_id, NetMsg::Bb(m)) => {
                    match m {
                        a => println!("{:?}", a)
                    }
                },
                _ => unreachable!()
            }
        }
    }
}

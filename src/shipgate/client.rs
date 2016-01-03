//! Client thread for shipgate connection.

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use std::net::SocketAddr;

use ::shipgate::msg::*;
use ::services::ServiceMsg;
use psoserial::Serial;

use std::net::TcpStream;

pub struct ShipGateClient {
    receiver: Receiver<ClientMsg>,
    stream: TcpStream,
    responders: HashMap<u32, Sender<ServiceMsg>>,
    response_counter: u32,
    password: String
}

enum ClientMsg {
    /// Send a message to the shipgate
    Send(Sender<ServiceMsg>, Message),
    // Respond to the shipgate.
    Recv(Message)
}

#[derive(Clone)]
pub struct ShipGateSender {
    tx: Sender<ClientMsg>,
    cb_sender: Option<Sender<ServiceMsg>>,
    req_counter: u32
}

impl ShipGateSender {
    pub fn send(&mut self, mut msg: Message) -> Result<u32, String> {
        match self.cb_sender {
            Some(ref cbs) => {
                msg.set_response_key(self.req_counter);
                self.req_counter += 1;
                self.tx.send(ClientMsg::Send(cbs.clone(), msg))
                    .map_err(|e| format!("{}", e)).map(|_| self.req_counter - 1)
            },
            None => Err("This sender does not have a callback sender specified.".to_string())
        }
    }

    pub fn clone_with(&self, cb_sender: Sender<ServiceMsg>) -> ShipGateSender {
        let mut r = self.clone();
        r.cb_sender = Some(cb_sender);
        r
    }
}

impl ShipGateClient {
    pub fn spawn(addr: SocketAddr, password: &str) -> ShipGateSender {
        let (tx, rx) = channel();

        let stream = TcpStream::connect(addr).unwrap();
        let s_c = stream.try_clone().unwrap();
        let pw = password.to_owned();
        thread::spawn(move|| {
            let c = ShipGateClient {
                receiver: rx,
                stream: s_c,
                responders: Default::default(),
                password: pw,
                response_counter: 0
            };
            c.run()
        });

        let tx_c = tx.clone();
        let mut s_c = stream.try_clone().unwrap();
        thread::spawn(move|| {
            loop {
                match Message::deserialize(&mut s_c) {
                    Ok(m) => {
                        if let Err(_) = tx_c.send(ClientMsg::Recv(m)) {
                            return
                        }
                    },
                    Err(_) => return
                }
            }
        });

        ShipGateSender {
            tx: tx,
            req_counter: 0,
            cb_sender: None
        }
    }

    pub fn run(mut self) {
        // Authenticate
        let m = Message::Auth(0, Auth(0, self.password.clone()));
        m.serialize(&mut self.stream).unwrap();

        for msg in self.receiver.iter() {
            match msg {
                ClientMsg::Send(callback, m) => {
                    self.responders.insert(self.response_counter, callback);
                    self.response_counter += 1;
                    m.serialize(&mut self.stream).unwrap();
                },
                ClientMsg::Recv(m) => {
                    let rk = m.get_response_key();
                    self.responders.get(&rk).map(|r| {
                        r.send(ServiceMsg::ShipGateMsg(m))
                    });
                }
            }
        }
    }
}

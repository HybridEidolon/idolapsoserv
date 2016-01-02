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
    tx: Sender<ClientMsg>
}

impl ShipGateSender {
    pub fn send(&mut self, callback: Sender<ServiceMsg>, msg: Message) -> Result<(), String> {
        self.tx.send(ClientMsg::Send(callback, msg)).map_err(|e| format!("{}", e))
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
                response_counter: 0,
                password: pw
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
            tx: tx
        }
    }

    pub fn run(mut self) {
        // Authenticate
        let m = Message::Auth(0, Auth(0, self.password.clone()));
        m.serialize(&mut self.stream).unwrap();

        for msg in self.receiver.iter() {
            match msg {
                ClientMsg::Send(callback, mut m) => {
                    m.set_response_key(self.response_counter);
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

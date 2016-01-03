//! Client thread for shipgate connection.

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ::shipgate::msg::*;
use ::services::ServiceMsg;
use psoserial::Serial;

use std::net::TcpStream;

pub mod callbacks;

pub struct ShipGateClient {
    receiver: Receiver<ClientMsg>,
    stream: TcpStream,
    responders: HashMap<u32, Sender<ServiceMsg>>,
    password: String
}

enum ClientMsg {
    /// Send a message to the shipgate
    Send(Sender<ServiceMsg>, Message),
    SendForget(Message),
    // Respond to the shipgate.
    Recv(Message)
}

#[derive(Clone)]
/// A wrapper holding a service message sender to send request responses to.
pub struct SgSender {
    tx: Sender<ClientMsg>,
    cb_sender: Option<Sender<ServiceMsg>>,
    req_counter: Arc<Mutex<u32>>
}

impl SgSender {
    /// Send a message, yielding a request number that the sender can record
    /// for response later.
    pub fn send(&mut self, mut msg: Message) -> Result<u32, String> {
        let k;
        if self.cb_sender.is_some() {
            k = try!(self.get_req_key());
            msg.set_response_key(k);
            self.tx.send(ClientMsg::Send(self.cb_sender.as_ref().unwrap().clone(), msg))
                .map_err(|e| format!("{}", e)).map(|_| k)
        } else {
            return Err("This sender does not have a callback sender specified".to_string())
        }
    }

    /// Send a message with no response code. The holder will not be told when
    /// a response is sent.
    pub fn send_forget(&mut self, msg: Message) -> Result<(), String> {
        self.tx.send(ClientMsg::SendForget(msg))
            .map_err(|e| format!("{}", e))
    }

    /// Clone this sender with the given service sender handle.
    pub fn clone_with(&self, cb_sender: Sender<ServiceMsg>) -> SgSender {
        SgSender {
            tx: self.tx.clone(),
            cb_sender: Some(cb_sender),
            req_counter: self.req_counter.clone()
        }
    }

    fn get_req_key(&mut self) -> Result<u32, String> {
        match self.req_counter.lock() {
            Ok(mut g) => {
                let v = *g;
                *g += 1;
                Ok(v)
            },
            Err(e) => Err(format!("{}", e))
        }
    }
}

impl ShipGateClient {
    pub fn spawn(addr: SocketAddr, password: &str) -> SgSender {
        let (tx, rx) = channel();

        let stream = TcpStream::connect(addr).unwrap();
        let s_c = stream.try_clone().unwrap();
        let pw = password.to_owned();
        thread::spawn(move|| {
            let c = ShipGateClient {
                receiver: rx,
                stream: s_c,
                responders: Default::default(),
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

        SgSender {
            tx: tx,
            req_counter: Arc::new(Mutex::new(1)),
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
                    self.responders.insert(m.get_response_key(), callback);
                    m.serialize(&mut self.stream).unwrap();
                },
                ClientMsg::SendForget(m) => {
                    m.serialize(&mut self.stream).unwrap();
                }
                ClientMsg::Recv(m) => {
                    let rk = m.get_response_key();
                    self.responders.get(&rk).map(|r| {
                        info!("Shipgate request had response callback: {:?}", m);
                        r.send(ServiceMsg::ShipGateMsg(m))
                    });
                }
            }
        }
    }
}

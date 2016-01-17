//! Service abstraction for the mio loop handler.

use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token};
use mio::util::Slab;

use std::io;
use std::sync::mpsc::Sender as MpscSender;
use std::net::SocketAddr;

pub mod client;
pub mod message;

use self::client::{Client, PatchClient, BbClient, ShipGateClient, ClientHandler};

use self::message::NetMsg;

use std::sync::Arc;

use ::shipgate::msg::Message as ShipGateMsg;

#[derive(Clone)]
pub enum ServiceMsg {
    ClientConnected((SocketAddr, usize)),
    ClientSaid(usize, NetMsg),
    ClientDisconnected(usize),
    ShipGateMsg(ShipGateMsg)
}

#[derive(Clone, PartialEq, Eq)]
pub enum ServiceType {
    /// Uses the Patch namespace in `psomsg::patch`
    Patch,
    /// Uses the Blue Burst namespace in `psomsg::bb`. 0 is the crypto key table
    Bb(Arc<Vec<u32>>),
    ShipGate
}

/// A communication handle for a service.
pub struct Service {
    pub listener: TcpListener,
    pub token: Token,
    clients: Slab<Client>,
    pub sender: MpscSender<ServiceMsg>,
    service_type: ServiceType
}

impl Service {
    pub fn new(listener: TcpListener, sender: MpscSender<ServiceMsg>, service_type: ServiceType) -> Service {
        Service {
            listener: listener,
            token: Token(0),
            clients: Slab::new(0),
            sender: sender,
            service_type: service_type
        }
    }

    pub fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        self.clients = Slab::new_starting_at(Token(self.token.0 * 10000), 2000);

        event_loop.register(
            &self.listener,
            self.token,
            EventSet::readable() | EventSet::hup(),
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        event_loop.reregister(
            &self.listener,
            self.token,
            EventSet::readable() | EventSet::hup(),
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn accept<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        let (sock, addr) = match self.listener.accept() {
            Ok(Some(s)) => {
                s
            },
            Ok(None) => {
                return self.reregister(event_loop)
            },
            Err(_e) => {
                return self.reregister(event_loop)
            }
        };

        // With the new socket, we now create a client for it and register it.
        let sender_clone = self.sender.clone();
        let st = self.service_type.clone();
        match self.clients.insert_with(|token| {
            match st {
                ServiceType::Patch => Client::Patch(PatchClient::new(sock, token, sender_clone)),
                ServiceType::Bb(ref kt) => Client::Bb(BbClient::new(sock, token, sender_clone, kt.clone())),
                ServiceType::ShipGate => Client::ShipGate(ShipGateClient::new(sock, token, sender_clone))
                //_ => unimplemented!()
            }
        }) {
            Some(token) => {
                // inserted successfully
                match self.get_client_mut(token).map(|c| c.register(event_loop)) {
                    Some(Ok(_)) => {
                        self.sender.send(ServiceMsg::ClientConnected((addr, token.0))).unwrap();
                    },
                    Some(Err(_e)) => {
                        self.clients.remove(token);
                    },
                    None => unreachable!() // maybe?
                }
            },
            None => {
                // failed to insert
            }
        }
        self.reregister(event_loop)
    }

    // pub fn get_client(&self, token: Token) -> Option<&Client> {
    //     self.clients.get(token)
    // }

    pub fn get_client_mut(&mut self, token: Token) -> Option<&mut Client> {
        self.clients.get_mut(token)
    }

    pub fn has_client(&self, token: Token) -> bool {
        self.clients.contains(token)
    }

    pub fn ready<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, token: Token, events: EventSet) {
        self.clients.get_mut(token).map(|c| {
            if events.contains(EventSet::readable()) {
                debug!("Reading from client token {}", token.0);
                c.readable(event_loop).unwrap();
            }
            if events.contains(EventSet::writable()) {
                debug!("Writing to client token {}", token.0);
                c.writable(event_loop).unwrap();
            }
        });
    }

    pub fn notify_svc<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, msg: ServiceMsg) {
        // Send the message on the channel to the appropriate thread.
        match self.sender.send(msg) {
            Err(_e) => event_loop.shutdown(),
            _ => ()
        }
    }

    pub fn notify_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, token: Token, msg: NetMsg) {
        self.clients.get_mut(token).map(|c| {
            c.send_msg(event_loop, msg)
        });
    }

    pub fn drop_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, token: Token) {
        self.clients.get_mut(token).map(|c| {
            c.drop_client(event_loop)
        });
        self.sender.send(ServiceMsg::ClientDisconnected(token.0)).unwrap();
        self.clients.remove(token);
    }
}

use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token};
use mio::util::Slab;

use std::io;
use std::sync::mpsc::Sender as MpscSender;

use psomsg::patch::Message;

mod client;
pub mod patch;

pub use self::client::Client;

#[derive(Clone)]
pub enum ServiceMsg {
    ClientConnected(usize),
    ClientSaid(usize, Box<Message>),
    ClientDisconnected(usize)
}

pub struct Service {
    pub listener: TcpListener,
    pub token: Token,
    clients: Slab<Client>,
    pub sender: MpscSender<ServiceMsg>
}

impl Service {
    pub fn new(listener: TcpListener, sender: MpscSender<ServiceMsg>) -> Service {
        use std::mem;

        Service {
            listener: listener,
            token: Token(0),
            clients: unsafe { mem::uninitialized() },
            sender: sender
        }
    }

    pub fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        self.clients = Slab::new_starting_at(Token(self.token.0 * 1000000), 2000);

        println!("registering service");

        event_loop.register(
            &self.listener,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        event_loop.reregister(
            &self.listener,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn accept<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) {
        let (sock, _addr) = match self.listener.accept() {
            Ok(Some(s)) => {
                s
            },
            Ok(None) => {
                self.reregister(event_loop).unwrap();
                return
            },
            Err(_e) => {
                self.reregister(event_loop).unwrap();
                return
            }
        };

        // With the new socket, we now create a client for it and register it.
        let sender_clone = self.sender.clone();
        match self.clients.insert_with(|token| {
            Client::new(sock, token, sender_clone)
        }) {
            Some(token) => {
                // inserted successfully
                match self.get_client_mut(token).map(|c| c.register(event_loop)) {
                    Some(Ok(_)) => {
                        println!("si seniorita");
                        self.sender.send(ServiceMsg::ClientConnected(token.0)).unwrap();
                    },
                    Some(Err(_e)) => {
                        println!("oh no friend :(");
                        self.clients.remove(token);
                    },
                    None => unreachable!() // maybe?
                }
            },
            None => {
                // failed to insert
            }
        }
        self.reregister(event_loop);
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
                c.readable(event_loop).unwrap();
            }
            if events.contains(EventSet::writable()) {
                c.writable(event_loop).unwrap();
            }
        });
    }

    pub fn notify_svc<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, msg: ServiceMsg) {
        // Send the message on the channel to the appropriate thread.
        match self.sender.send(msg) {
            Err(e) => event_loop.shutdown(),
            _ => ()
        }
    }

    pub fn notify_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, token: Token, msg: Box<Message>) {
        self.clients.get_mut(token).map(|c| {
            c.send_msg(event_loop, msg)
        });
    }
}

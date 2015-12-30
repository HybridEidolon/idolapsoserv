use mio::{Handler, EventLoop, Token, EventSet};
use mio::util::Slab;

use ::services::{Service, ServiceMsg};

use psomsg::patch::Message;

#[derive(Clone)]
pub enum LoopMsg {
    /// Send a system message to a service.
    Service(Token, ServiceMsg),

    /// Send a message to a client.
    Client(usize, Box<Message>)
}

pub struct LoopHandler {
    services: Slab<Service>
}

impl LoopHandler {
    pub fn new<H: Handler>(services: Vec<Service>, event_loop: &mut EventLoop<H>) -> LoopHandler {
        let mut svcs = Slab::new_starting_at(Token(1), 16);
        for mut s in services {
            svcs.insert_with(|token| {
                s.token = token;
                s
            });
        }

        let mut r = LoopHandler {
            services: svcs
        };

        for s in r.services.iter_mut() {
            s.register(event_loop).unwrap();
        }

        r
    }
}

impl Handler for LoopHandler {
    type Timeout = usize;
    type Message = LoopMsg;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        match self.services.get_mut(token) {
            Some(s) => {
                // Accept
                match s.accept(event_loop) {
                    Err(_e) => event_loop.shutdown(),
                    Ok(_) => ()
                }
                return
            },
            None => {
                // This token is not a service, but a connected client.
                // We will continue below.
            }
        }

        // The token wasn't a service, so pass this ready event on to the
        // service it DOES belong to.
        self.services.iter_mut()
            .find(|s| s.has_client(token))
            .map(|s| s.ready(event_loop, token, events))
            .unwrap_or_else(|| {
                error!("token {:?} is not a valid client or service", token);
            });
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: LoopMsg) {
        match msg {
            LoopMsg::Service(t, m) => {
                self.services.get_mut(t)
                    .map(|s| s.notify_svc(event_loop, m))
                    .unwrap_or_else(|| {
                        error!("we don't have a service on token {:?}...", t);
                    });
            },
            LoopMsg::Client(t, m) => {
                self.services.iter_mut()
                    .find(|s| s.has_client(Token(t)))
                    .map(|s| {
                        s.notify_client(event_loop, Token(t), m);
                    })
                    .unwrap_or_else(|| {
                        error!("no client?")
                    });
            }
        }
    }

    // fn timeout(&mut self, _event_loop: &mut EventLoop<Self>, _timeout: Self::Timeout) {
    //
    // }

    fn interrupted(&mut self, event_loop: &mut EventLoop<Self>) {
        info!("Interrupted");
        event_loop.shutdown();
    }

    // fn tick(&mut self, _event_loop: &mut EventLoop<Self>) {
    //
    // }
}

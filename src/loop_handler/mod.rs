use mio::{Handler, EventLoop, Token, EventSet};
use mio::util::Slab;

use ::services::{Service, ServiceMsg};

use ::services::message::NetMsg;

#[derive(Clone)]
pub enum LoopMsg {
    /// Send a system message to a service.
    Service(Token, ServiceMsg),

    /// Send a message to a client.
    Client(usize, NetMsg),

    /// Drop a client
    DropClient(usize)
}

// impl From<(usize, NetMsg)> for LoopMsg {
//     #[inline(always)]
//     fn from(val: (usize, NetMsg)) -> LoopMsg {
//         LoopMsg::Client(val.0, val.1)
//     }
// }

impl<I: Into<NetMsg>> From<(usize, I)> for LoopMsg {
    #[inline(always)]
    fn from(val: (usize, I)) -> LoopMsg {
        LoopMsg::Client(val.0, val.1.into())
    }
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
        debug!("Ready");
        if events.contains(EventSet::readable()) {
            match self.services.get_mut(token) {
                Some(s) => {
                    // Accept
                    debug!("Listener accept");
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
        }

        if events.contains(EventSet::readable()) || events.contains(EventSet::writable()) {
            // The token wasn't a service, so pass this ready event on to the
            // client in the service it belongs to.
            debug!("Client stream readable or writable");
            self.services.iter_mut()
                .find(|s| s.has_client(token))
                .map(|s| s.ready(event_loop, token, events))
                .unwrap_or_else(|| {
                    error!("token {:?} is not a valid client or service", token);
                });
        }

        if events.contains(EventSet::hup()) {
            debug!("Token {} hup", token.0);
            match self.services.iter_mut().find(|s| s.has_client(token)) {
                Some(s) => s.drop_client(event_loop, token),
                None => {
                    // this is a service hupping, shutdown
                    warn!("A service listener got hup, shutting down loop.");
                    event_loop.shutdown();
                    return
                }
            }
        }

    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: LoopMsg) {
        debug!("Notify");
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
            },
            LoopMsg::DropClient(t) => {
                self.services.iter_mut()
                    .find(|s| s.has_client(Token(t)))
                    .map(|s| {
                        s.drop_client(event_loop, Token(t));
                    })
                    .unwrap_or_else(|| {
                        error!("attempted to drop client {} but doesn't exist", t)
                    });
            }
        }
    }

    fn timeout(&mut self, _event_loop: &mut EventLoop<Self>, _timeout: Self::Timeout) {
        debug!("Timeout triggered");
    }

    fn interrupted(&mut self, event_loop: &mut EventLoop<Self>) {
        info!("Interrupted");
        event_loop.shutdown();
    }

    fn tick(&mut self, _event_loop: &mut EventLoop<Self>) {
        debug!("Mio Loop Tick");
    }
}

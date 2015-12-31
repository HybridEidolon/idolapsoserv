//! Client stream handlers for the mio event loop. Spawned by the Service
//! accept handler. Each one parses messages into their respective message
//! namespace.

pub mod patch;
pub mod bb;

pub use self::patch::PatchClient;
pub use self::bb::BbClient;

use mio::{EventLoop, Handler};
use std::io;

use ::services::message::NetMsg;

/// Pads a number to a certain multiple.
#[inline(always)]
pub fn padded(value: usize, multiple: usize) -> usize {
    if value % multiple != 0 {
        value + (multiple - value % multiple)
    } else {
        value
    }
}

pub trait ClientHandler {
    type Msg;
    fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()>;
    fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()>;
    fn readable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()>;
    fn writable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()>;
    fn send_msg<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, msg: Self::Msg) -> io::Result<()>;
    fn drop_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()>;
}

pub enum Client {
    Patch(PatchClient),
    Bb(BbClient)
}

impl ClientHandler for Client {
    type Msg = NetMsg;
    fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => p.register(event_loop),
            &mut Client::Bb(ref mut b) => b.register(event_loop)
        }
    }

    fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => p.reregister(event_loop),
            &mut Client::Bb(ref mut b) => b.reregister(event_loop)
        }
    }

    fn readable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => p.readable(event_loop),
            &mut Client::Bb(ref mut b) => b.readable(event_loop)
        }
    }

    fn writable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => p.writable(event_loop),
            &mut Client::Bb(ref mut b) => b.writable(event_loop)
        }
    }

    fn send_msg<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, msg: Self::Msg) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => match msg {
                NetMsg::Patch(m) => p.send_msg(event_loop, m),
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid message type sent to Patch client"))
            },
            &mut Client::Bb(ref mut b) => match msg {
                NetMsg::Bb(m) => b.send_msg(event_loop, m),
                _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid message type sent to BB client"))
            }
        }
    }

    fn drop_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        match self {
            &mut Client::Patch(ref mut p) => p.drop_client(event_loop),
            &mut Client::Bb(ref mut b) => b.drop_client(event_loop)
        }
    }
}

use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token};

use std::io;
use std::sync::mpsc::Sender as MpscSender;

use psocrypto::PcCipher;
use psomsg::patch::Message;

use ::services::ServiceMsg;

pub struct Client {
    pub stream: TcpStream,
    pub token: Token,
    pub client_cipher: Option<PcCipher>,
    pub server_cipher: Option<PcCipher>,
    //welcomed: bool,
    interests: EventSet,
    sender: MpscSender<ServiceMsg>
}

impl Client {
    pub fn new(stream: TcpStream, token: Token, thread_sender: MpscSender<ServiceMsg>) -> Client {
        Client {
            stream: stream,
            token: token,
            client_cipher: None,
            server_cipher: None,
            //welcomed: false,
            interests: EventSet::none(),
            sender: thread_sender
        }
    }

    pub fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        // We will probably want to know when to read first.
        // The event loop notify() will tell us when we want to write.
        self.interests.insert(EventSet::readable());

        event_loop.register(
            &self.stream,
            self.token,
            self.interests,
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        event_loop.reregister(
            &self.stream,
            self.token,
            self.interests,
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    pub fn readable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        // Do nothing; this is unimplemented
        // At some point, read into a buffer the header, then the body, then
        // send a message on the service sender.
        self.interests.insert(EventSet::readable());
        self.reregister(event_loop)
    }

    pub fn writable<H: Handler>(&mut self, _event_loop: &mut EventLoop<H>) -> io::Result<()> {
        // Do nothing; unimplemented
        // This will pop a message from the message queue, encode and encrypt it,
        // then continue attempting to write to the socket.
        Ok(())
    }

    pub fn send_msg<H: Handler>(&mut self, _event_loop: &mut EventLoop<H>, _msg: Box<Message>) {
        // Do nothing; unimplemented
        unimplemented!()
    }
}

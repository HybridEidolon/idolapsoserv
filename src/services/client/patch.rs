use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token, TryRead, TryWrite};

use std::io;
use std::sync::mpsc::Sender as MpscSender;
use std::collections::VecDeque;

use psocrypto::{PcCipher, Encryptor, Decryptor};
use psomsg::patch::Message;
use psomsg::Serial;

use ::services::ServiceMsg;

use ::services::message::NetMsg;

use super::{padded, ClientHandler};

#[derive(Clone, Copy)]
enum SendState {
    WaitingForMsg,
    SendingMsg(usize)
}

impl Default for SendState {
    #[inline(always)]
    fn default() -> SendState { SendState::WaitingForMsg }
}

#[derive(Clone, Copy)]
enum ReadState {
    ReadingHdr(usize),
    ReadingBody(usize, usize)
}

impl Default for ReadState {
    #[inline(always)]
    fn default() -> ReadState { ReadState::ReadingHdr(0) }
}

pub struct PatchClient {
    pub stream: TcpStream,
    pub token: Token,
    pub ciphers: Option<(PcCipher, PcCipher)>,
    interests: EventSet,
    sender: MpscSender<ServiceMsg>,
    send_queue: VecDeque<Message>,
    send_state: SendState,
    read_state: ReadState,
    send_buffer: Vec<u8>,
    read_buffer: Vec<u8>,
}

impl PatchClient {
    pub fn new(stream: TcpStream, token: Token, thread_sender: MpscSender<ServiceMsg>) -> PatchClient {
        PatchClient {
            stream: stream,
            token: token,
            ciphers: None,
            interests: EventSet::none(),
            sender: thread_sender,
            send_queue: VecDeque::with_capacity(8),
            send_state: SendState::WaitingForMsg,
            send_buffer: Vec::new(),
            read_state: Default::default(),
            read_buffer: vec![0; 4096]
        }
    }
}
impl ClientHandler for PatchClient {
    type Msg = Message;
    fn register<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
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

    fn reregister<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        event_loop.reregister(
            &self.stream,
            self.token,
            self.interests,
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    fn readable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        use std::io::Cursor;
        // Do nothing; this is unimplemented
        // At some point, read into a buffer the header, then the body, then
        // send a message on the service sender.
        loop { match self.read_state {
            ReadState::ReadingHdr(start) => {
                debug!("Reading header");
                let remaining = 4 - start;
                match if remaining != 0 {self.stream.try_read(&mut self.read_buffer[start..4])} else {Ok(Some(0))} {
                    Ok(Some(bytes)) => {
                        if bytes < remaining {
                            debug!("Header incomplete");
                            // still need to read more, return and resume later
                            self.read_state = ReadState::ReadingHdr(start + bytes);
                            self.interests.insert(EventSet::readable());
                            return self.reregister(event_loop)
                        } else {
                            debug!("Header complete");
                            // buffer is filled
                            // decrypt if necessary
                            if let Some((_, ref mut c)) = self.ciphers {
                                debug!("Decrypting");
                                c.decrypt_in_place(&mut self.read_buffer[0..4]).unwrap();
                            }
                            // little endian u16 for size
                            debug!("Hdr Buffer is: {:?}", &self.read_buffer[0..4]);
                            let size;
                            {
                                use byteorder::{LittleEndian as LE, ReadBytesExt};
                                size = try!(Cursor::new(&self.read_buffer[..]).read_u16::<LE>()) as usize;
                            }
                            let padded_size = padded(size, 4);
                            let buffer_len = self.read_buffer.len();
                            if buffer_len < padded_size {
                                // need to expand the read buffer
                                self.read_buffer.append(&mut vec![0; padded_size - buffer_len]);
                                // we'll reuse this buffer later so it probably won't resize much
                            }
                            self.read_state = ReadState::ReadingBody(4, size)
                            // loop back to ReadingBody
                        }
                    },
                    Ok(None) => {
                        debug!("No bytes were read because it would block");
                        self.read_state = ReadState::ReadingHdr(start);
                        self.interests.insert(EventSet::readable());
                        return self.reregister(event_loop)
                    }
                    Err(e) => {
                        return Err(e)
                    }
                }
            },
            ReadState::ReadingBody(start, size) => {
                debug!("Reading body");
                let padded_size = padded(size, 4);
                let remaining = padded_size - start;
                debug!("{} Remaining bytes", remaining);
                match if remaining != 0 {self.stream.try_read(&mut self.read_buffer[start..padded_size])} else {Ok(Some(0))} {
                    Ok(Some(bytes)) => {
                        if bytes < remaining {
                            debug!("Body incomplete; read {}", bytes);
                            // still need to read more, return and resume later
                            self.read_state = ReadState::ReadingBody(start + bytes, size);
                            self.interests.insert(EventSet::readable());
                            return self.reregister(event_loop)
                        } else {
                            debug!("Body complete");
                            // buffer is filled
                            // decrypt if necessary
                            if let Some((_, ref mut c)) = self.ciphers {
                                c.decrypt_in_place(&mut self.read_buffer[4..size]).unwrap();
                            }
                            // parse into message
                            let message = try!(Message::deserialize(&mut Cursor::new(&self.read_buffer[0..size])));
                            // send message to service thread
                            match self.sender.send(ServiceMsg::ClientSaid(self.token.0, NetMsg::Patch(message))) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("Failed to send client message to service thread.");
                                    event_loop.shutdown();
                                    return Err(io::Error::new(io::ErrorKind::Other, e))
                                }
                            }

                            self.read_state = ReadState::ReadingHdr(0);
                            // loop back to ReadingHdr
                        }
                    },
                    Ok(None) => {
                        debug!("No bytes were read because it would block");
                        self.read_state = ReadState::ReadingBody(start, size);
                        self.interests.insert(EventSet::readable());
                        return self.reregister(event_loop)
                    }
                    Err(e) => return Err(e)
                }
            }
        }}
    }

    fn writable<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        use psomsg::Serial;
        use std::io::Cursor;
        use std::mem::swap;
        use psomsg::patch::*;

        // holy state machines batman
        loop { match self.send_state {
            SendState::WaitingForMsg => {
                match self.send_queue.pop_front() {
                    Some(msg) => {
                        let mut buf = Vec::new();
                        swap(&mut buf, &mut self.send_buffer);
                        buf.clear();
                        let mut c = Cursor::new(buf);
                        msg.serialize(&mut c).unwrap();
                        let mut buf = c.into_inner();
                        if let Some((ref mut s, _)) = self.ciphers {
                            // encrypt if we have ciphers
                            s.encrypt_in_place(&mut buf[..]).unwrap();
                        }

                        // special case: it's a Welcome msg
                        // we prep our internal ciphers
                        if let &Message::Welcome(Some(
                            Welcome {
                                ref server_vector,
                                ref client_vector
                            })) = &msg {
                                self.ciphers = Some((
                                    PcCipher::new(*server_vector),
                                    PcCipher::new(*client_vector)
                                ));
                        }

                        self.send_state = SendState::SendingMsg(0);
                        swap(&mut buf, &mut self.send_buffer);
                        // we'll loop again to the SendingMsg handler.
                    },
                    None => {
                        // Nothing in the queue; remove writing from our interests
                        // until we're notified again.
                        self.interests.remove(EventSet::writable());
                        return self.reregister(event_loop)
                    }
                }
            },
            SendState::SendingMsg(start) => {
                // now, try sending the contents of this buffer.
                match self.stream.try_write(&self.send_buffer[start..]) {
                    Ok(Some(bytes)) => {
                        if bytes < self.send_buffer.len() {
                            // Socket was not ready to send whole message,
                            // resume on next writable.
                            self.interests.insert(EventSet::writable());
                            self.send_state = SendState::SendingMsg(start + bytes);
                            return self.reregister(event_loop)
                        } else {
                            self.send_state = SendState::WaitingForMsg
                            // loop back around... it will return if there's
                            // no messages left.
                        }
                    },
                    Ok(None) => {
                        debug!("No bytes were written because it would block");
                        self.interests.insert(EventSet::writable());
                        self.send_state = SendState::SendingMsg(start);
                        return self.reregister(event_loop)
                    }
                    Err(e) => {
                        return Err(e)
                    }
                }
            }
        } }
    }

    fn send_msg<H: Handler>(&mut self, event_loop: &mut EventLoop<H>, msg: Message) -> io::Result<()> {
        self.send_queue.push_back(msg);
        self.interests.insert(EventSet::writable());
        self.reregister(event_loop)
    }

    fn drop_client<H: Handler>(&mut self, event_loop: &mut EventLoop<H>) -> io::Result<()> {
        // unregister self; service will remove token and stream from slab
        event_loop.deregister(&self.stream)
    }
}

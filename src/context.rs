//! Trait for representing client connection contexts.

use std::io;
use std::io::{Read, Write};

use psocrypto::{Encryptor, Decryptor};

use psomsg::{MessageEncode, MessageDecode};

/// A trait representing a client connection context. You can send and receive messages on this
/// interface using the context's provided IO endpoints and ciphers.
pub trait Context {
    /// Send a message unencrypted.
    #[inline]
    fn send_msg_unenc(&mut self, msg: &MessageEncode) -> io::Result<()> {
        let (writer, _) = try!(self.get_write_encryptor());
        try!(msg.encode_msg(writer, None));
        writer.flush()
    }

    /// Send a message.
    #[inline]
    fn send_msg(&mut self, msg: &MessageEncode) -> io::Result<()> {
        let (writer, encryptor) = try!(self.get_write_encryptor());
        try!(msg.encode_msg(writer, Some(encryptor)));
        writer.flush()
    }

    /// Send raw bytes.
    #[inline]
    fn send_raw(&mut self, _: &[u8]) -> io::Result<()> {
        unimplemented!()
    }

    /// Receive a message based on the template T. Will defer to the decoding implementation of
    /// T to receive from the context.
    #[inline]
    fn recv_msg<T: MessageDecode>(&mut self) -> io::Result<T> {
        let (read, decryptor) = try!(self.get_read_decryptor());
        T::decode_msg(read, Some(decryptor))
    }
    /// Receive raw bytes into a buffer.
    #[inline]
    fn recv(&mut self, _: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }

    /// Get writable endpoint and encryptor.
    fn get_write_encryptor(&mut self) -> io::Result<(&mut Write, &mut Encryptor)>;
    /// Get readable endpoint and encryptor.
    fn get_read_decryptor(&mut self) -> io::Result<(&mut Read, &mut Decryptor)>;
}

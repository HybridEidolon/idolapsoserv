//! Message structures for Blue Burst.

#![allow(unused_variables)]

use std::io::{Read, Write};
use std::io;

use std::net::Ipv4Addr;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

pub static PSOBB_COPYRIGHT_STRING: &'static [u8] = b"Phantasy Star Online Blue Burst Game Server. Copyright 1999-2004 SONICTEAM.";

pub trait Serial: Sized {
    fn serialize(&self, dst: &mut Write) -> io::Result<()>;
    fn deserialize(src: &mut Read) -> io::Result<Self>;
}

macro_rules! impl_unit_serial {
    ($name:ident) => {
        impl Serial for $name {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                Ok(())
            }
            fn deserialize(src: &mut Read) -> io::Result<Self> {
                Ok($name)
            }
        }
    }
}

macro_rules! gen_message_enum {
    ($($id:expr => $name:ident),*) => {
        #[derive(Clone, Debug)]
        pub enum Message {
            Unknown(u16, u32, Vec<u8>),
            $($name(u32, $name)),*
        }

        impl Serial for Message {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                use std::io::Cursor;
                let mut buf = Vec::with_capacity(512);
                let msg_type: u16;
                let size: u16;
                let flags: u32;
                debug!("Serializing message");
                {
                    let mut cur = Cursor::new(buf);
                    match self {
                        &Message::Unknown(ref a, ref f, ref bytes) => {
                            try!(cur.write_all(&bytes[..]));
                            msg_type = *a;
                            flags = *f;
                        },
                        $(&Message::$name(ref f, ref a) => {
                            try!(a.serialize(&mut cur));
                            msg_type = $id as u16;
                            flags = *f;
                        }),*
                    };
                    size = cur.position() as u16;
                    buf = cur.into_inner();
                }
                debug!("Serializing header");
                let hdr_buf;
                {
                    let mut curs = Cursor::new(Vec::with_capacity(8));
                    try!(curs.write_u16::<LE>(size + 8));
                    try!(curs.write_u16::<LE>(msg_type));
                    try!(curs.write_u32::<LE>(flags));
                    hdr_buf = curs.into_inner();
                }
                debug!("Serializing into: size {}, msg_type {}, flags {}", size, msg_type, flags);
                try!(dst.write_all(&hdr_buf));
                debug!("Serializing message contents");
                if buf.len() % 8 != 0 {
                    let buf_len = buf.len();
                    debug!("contents need to be padded by {}", 8 - buf_len % 8);
                    buf.append(&mut vec![0u8; (8 - buf_len % 8) as usize]);
                }
                try!(dst.write_all(&buf));

                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<Self> {
                use std::io::Cursor;
                // parse header
                let mut hdr_buf = vec![0u8; 8];
                debug!("Reading message header");
                if try!(src.read(&mut hdr_buf)) != 8 {
                    return Err(io::Error::new(io::ErrorKind::Other, "unexpected EOF parsing header"))
                }
                let size;
                let msg_type;
                let flags;
                {
                    debug!("Parsing message header");
                    let mut hdr_cursor = Cursor::new(hdr_buf);
                    size = try!(hdr_cursor.read_u16::<LE>());
                    msg_type = try!(hdr_cursor.read_u16::<LE>());
                    flags = try!(hdr_cursor.read_u32::<LE>());
                }
                debug!("size: {size}, type: {msg_type}, flags: {flags}", size=size, msg_type=msg_type, flags=flags);
                if size % 8 > 0 {
                    warn!("message size {} is not a multiple of 8", size);
                    // Blue Burst sends the padded zeros with the message. We need to read them
                    // and consider them for decryption.
                    // In unencrypted mode, they can simply be ignored by the parser and nothing
                    // bad will occur.
                }
                let mut msg_buf = vec![0u8; (size + 8 - (size % 8)) as usize - 8];
                if try!(src.read(&mut msg_buf)) != (size + 8 - (size % 8)) as usize - 8 {
                    return Err(io::Error::new(io::ErrorKind::Other, "unexpected EOF getting rest of message"))
                }
                let mut msg_cur = Cursor::new(msg_buf);
                match msg_type {
                    $($id => Ok(Message::$name(flags, try!($name::deserialize(&mut msg_cur)))),)*
                    a => {
                        Ok(Message::Unknown(a, flags, msg_cur.into_inner()))
                    }
                }
            }
        }
    }
}

gen_message_enum! {
    0x0003 => Welcome,
    0x0019 => Redirect,
    0x0093 => Login,
    0x00E6 => BbSecurity
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Welcome(pub Vec<u8>, pub Vec<u8>);
impl Serial for Welcome {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        let padding = vec![0u8; 0x60 - PSOBB_COPYRIGHT_STRING.len()];
        assert_eq!(0x60, PSOBB_COPYRIGHT_STRING.len() + padding.len());
        try!(dst.write_all(PSOBB_COPYRIGHT_STRING));
        try!(dst.write_all(&padding[..]));
        try!(dst.write_all(&self.0[..]));
        try!(dst.write_all(&self.1[..]));
        Ok(())
    }
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

fn read_ascii(src: &mut Read, len: u32) -> io::Result<String> {
    use encoding::all::ASCII;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    // up to first null
    let mut end = 0;
    {
        for (i, c) in r.iter().enumerate() {
            if *c == 0 {
                end = i;
                break;
            }
        }
    }
    match ASCII.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode ascii: {:?}", e)))
    }
}

fn read_array(src: &mut Read, len: u32) -> io::Result<Vec<u8>> {
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    Ok(r)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Login {
    pub tag: u32,
    pub guildcard: u32,
    pub version: u16,
    pub unk: Vec<u8>, // 6
    pub team_id: u32,
    pub username: String, // 32
    //pub unused1: [u8; 32],
    pub password: String, // 32
    //pub unused2: [u8; 40],
    pub hw_info: Vec<u8>, // 8
    pub security_data: Vec<u8> // 40
}
impl Serial for Login {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let tag = try!(src.read_u32::<LE>());
        let guildcard = try!(src.read_u32::<LE>());
        let version = try!(src.read_u16::<LE>());
        let unk = try!(read_array(src, 6));
        let team_id = try!(src.read_u32::<LE>());
        let username = try!(read_ascii(src, 16));
        try!(src.read(&mut [0u8; 32]));
        let password = try!(read_ascii(src, 16));
        try!(src.read(&mut [0u8; 40]));
        let hw_info = try!(read_array(src, 8));
        let security_data = try!(read_array(src, 40));
        Ok(Login {
            tag: tag,
            guildcard: guildcard,
            version: version,
            unk: unk,
            team_id: team_id,
            username: username,
            password: password,
            hw_info: hw_info,
            security_data: security_data
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbSecurity {
    // bb_pkt_hdr_t hdr;
    // uint32_t err_code;
    // uint32_t tag;
    // uint32_t guildcard;
    // uint32_t team_id;
    // uint8_t security_data[40];
    // uint32_t caps;
    pub err_code: u32,
    pub tag: u32,
    pub guildcard: u32,
    pub team_id: u32,
    pub security_data: BbSecurityData,
    pub caps: u32
}
impl Serial for BbSecurity {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.err_code));
        try!(dst.write_u32::<LE>(self.tag));
        try!(dst.write_u32::<LE>(self.guildcard));
        try!(dst.write_u32::<LE>(self.team_id));
        try!(self.security_data.serialize(dst));
        try!(dst.write_u32::<LE>(self.caps));
        Ok(())
    }
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Redirect {
    // uint32_t ip_addr;       /* Big-endian */
    // uint16_t port;          /* Little-endian */
    // uint8_t padding[2];
    pub ip: Ipv4Addr,
    pub port: u16
}

impl Serial for Redirect {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.ip.serialize(dst));
        try!(dst.write_u16::<LE>(self.port));
        try!(dst.write_u16::<LE>(0)); // padding
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

// Other Serial implementations
impl Serial for Ipv4Addr {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        let octets = self.octets();
        try!(dst.write_all(&octets[..]));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BbSecurityData {
    // uint32_t magic;                     /* Must be 0xDEADBEEF */
    // uint8_t slot;                       /* Selected character */
    // uint8_t sel_char;                   /* Have they selected a character? */
    // uint8_t reserved[34];               /* Set to 0 */
    pub magic: u32,
    pub slot: u8,
    pub sel_char: bool
}

impl Serial for BbSecurityData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.magic));
        try!(dst.write_u8(self.slot));
        try!(dst.write_u8(if self.sel_char {1} else {0}));
        try!(dst.write_all(&[0; 34][..]));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let magic = try!(src.read_u32::<LE>());
        let slot = try!(src.read_u8());
        let sel_char = match try!(src.read_u8()) { 0 => false, _ => true };
        try!(src.read_exact(&mut [0; 34][..]));
        Ok(BbSecurityData {
            magic: magic,
            slot: slot,
            sel_char: sel_char
        })
    }
}

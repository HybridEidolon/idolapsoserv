//! Message structures for Blue Burst.

#![allow(unused_variables)]

use std::io::{Read, Write};
use std::io;

use std::net::Ipv4Addr;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

pub static PSOBB_COPYRIGHT_STRING: &'static [u8] = b"Phantasy Star Online Blue Burst Game Server. Copyright 1999-2004 SONICTEAM.";

pub mod default_config;

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
                let mut buf = Vec::with_capacity(4096);
                let msg_type: u16;
                let mut size: u16;
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
                if buf.len() % 8 != 0 {
                    let buf_len = buf.len();
                    debug!("contents need to be padded by {}", 8 - buf_len % 8);
                    buf.append(&mut vec![0u8; (8 - buf_len % 8) as usize]);
                    size += (8 - buf_len % 8) as u16;
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
                debug!("Serializing into: size {}, msg_type {:x}, flags {}", size, msg_type, flags);
                try!(dst.write_all(&hdr_buf));
                debug!("Serializing message contents");
                try!(dst.write_all(&buf));

                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<Self> {
                use std::io::Cursor;
                // parse header
                let mut hdr_buf = vec![0u8; 8];
                debug!("Reading message header");
                if try!(src.read(&mut hdr_buf[..])) != 8 {
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
                debug!("size: {size}, type: {msg_type:x}, flags: {flags}", size=size, msg_type=msg_type, flags=flags);

                let padding = if size % 8 == 0 { 0 } else { 8 - (size % 8) };

                let mut msg_buf = vec![0u8; (size + padding) as usize - 8];
                if size > 8 {
                    if try!(src.read(&mut msg_buf)) != (size + padding) as usize - 8 {
                        return Err(io::Error::new(io::ErrorKind::Other, "unexpected EOF getting rest of message"))
                    }
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
    0x0005 => Goodbye,
    0x0019 => Redirect,
    0x0093 => Login,
    0x001A => LargeMsg,
    0x01DC => BbGuildCardHdr,
    0x02DC => BbGuildCardChunk,
    0x03DC => BbGuildCardChunkReq,
    0x00E0 => BbOptionRequest,
    0x00E2 => BbOptionConfig,
    0x00E3 => BbCharSelect,
    0x00E4 => BbCharAck,
    0x00E6 => BbSecurity,
    0x01E8 => BbChecksum,
    0x02E8 => BbChecksumAck,
    0x03E8 => BbGuildRequest,
    0x04E8 => BbAddGuildCard,
    0x01EB => BbParamHdr,
    0x02EB => BbParamChunk,
    0x03EB => BbParamChunkReq,
    0x04EB => BbParamHdrReq,
    0x00EC => BbSetFlags
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

fn write_ascii_len(s: &str, len: usize, dst: &mut Write) -> io::Result<()> {
    use encoding::all::ASCII;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match ASCII.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode ASCII: {:?}", e)))
    };

    let padding: isize = len as isize - r.len() as isize;
    if padding < 0 {
        warn!("utf16 string too long, truncating to fit");
        try!(dst.write_all(&r[..len]));
        Ok(())
    } else {
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

fn write_utf16(s: &str, dst: &mut Write) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };
    try!(dst.write_all(&r[..]));
    Ok(())
}

fn read_utf16_len(len: usize, src: &mut Read) -> io::Result<String> {
    use encoding::all::UTF_16LE;
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
    match UTF_16LE.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode utf16: {:?}", e)))
    }
}

fn write_utf16_len(s: &str, len: usize, dst: &mut Write) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };

    let padding: isize = len as isize - r.len() as isize;
    if padding < 0 {
        warn!("utf16 string too long, truncating to fit");
        try!(dst.write_all(&r[..len]));
        Ok(())
    } else {
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

fn read_array(src: &mut Read, len: u32) -> io::Result<Vec<u8>> {
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    Ok(r)
}

fn write_array(sl: &[u8], len: u32, dst: &mut Write) -> io::Result<()> {
    if sl.len() > len as usize {
        warn!("Slice is too big to fit in buffer, writing truncated");

        Ok(())
    } else {
        let padding = len as i32 - sl.len() as i32;
        try!(dst.write_all(&sl[..]));
        if padding > 0 {
            try!(dst.write_all(&vec![0u8; padding as usize][..]));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Goodbye;
impl_unit_serial!(Goodbye);

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LargeMsg(pub String);
impl Serial for LargeMsg {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbOptionRequest;
impl_unit_serial!(BbOptionRequest);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbOptionConfig(pub BbTeamAndKeyData);
impl Serial for BbOptionConfig {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(BbTeamAndKeyData::deserialize(src));
        Ok(BbOptionConfig(data))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbCharSelect {
    pub slot: u32, // This is actually ranged [0,5)
    pub selecting: bool
}
impl Serial for BbCharSelect {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let slot = try!(src.read_u32::<LE>());
        let selecting = match try!(src.read_u32::<LE>()) { 0 => false, _ => true };
        Ok(BbCharSelect {
            slot: slot,
            selecting: selecting
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbCharAck {
    pub slot: u32,
    pub code: u32
}
impl Serial for BbCharAck {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.slot));
        try!(dst.write_u32::<LE>(self.code));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let slot = try!(src.read_u32::<LE>());
        let code = try!(src.read_u32::<LE>());
        Ok(BbCharAck {
            slot: slot,
            code: code
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbChecksum(pub u32);
impl Serial for BbChecksum {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.0));
        try!(dst.write_u32::<LE>(0));
        Ok(())
    }
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let cs = try!(src.read_u32::<LE>());
        try!(src.read_u32::<LE>());
        Ok(BbChecksum(cs))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbChecksumAck(pub bool);
impl Serial for BbChecksumAck {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(if self.0 { 1 } else { 0 }));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let b = try!(src.read_u32::<LE>());
        Ok(BbChecksumAck(match b { 0 => false, _ => true }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbGuildCardChunkReq(pub u32, pub u32, pub bool);
impl Serial for BbGuildCardChunkReq {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let unk = try!(src.read_u32::<LE>());
        let chunk = try!(src.read_u32::<LE>());
        let cont = match try!(src.read_u32::<LE>()) { 0 => false, _ => true };
        Ok(BbGuildCardChunkReq(unk, chunk, cont))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbGuildRequest;
impl_unit_serial!(BbGuildRequest);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbGuildCardHdr {
    pub one: u32,
    pub len: u32,
    pub checksum: u32
}
impl Serial for BbGuildCardHdr {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.one));
        try!(dst.write_u32::<LE>(self.len));
        try!(dst.write_u32::<LE>(self.checksum));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let one = try!(src.read_u32::<LE>());
        let len = try!(src.read_u32::<LE>());
        let checksum = try!(src.read_u32::<LE>());
        Ok(BbGuildCardHdr {
            one: one,
            len: len,
            checksum: checksum
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbGuildCardChunk {
    pub unk: u32,
    pub chunk: u32,
    pub data: Vec<u8>
}
impl Serial for BbGuildCardChunk {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.unk));
        try!(dst.write_u32::<LE>(self.chunk));
        try!(dst.write_all(&self.data[..]));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbAddGuildCard {
    // bb_pkt_hdr_t hdr;
    // uint32_t guildcard;
    // uint16_t name[24];
    // uint16_t team_name[16];
    // uint16_t text[88];
    // uint8_t one;
    // uint8_t language;
    // uint8_t section;
    // uint8_t char_class;
    pub guildcard: u32,
    pub name: String,
    pub team_name: String,
    pub text: String,
    pub one: u8,
    pub lang: u8,
    pub section: u8,
    pub char_class: u8
}
impl Serial for BbAddGuildCard {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        unimplemented!()
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let guildcard = try!(src.read_u32::<LE>());
        let name = try!(read_utf16_len(48, src));
        let team_name = try!(read_utf16_len(32, src));
        let text = try!(read_utf16_len(88*2, src));
        let one = try!(src.read_u8());
        let lang = try!(src.read_u8());
        let section = try!(src.read_u8());
        let char_class = try!(src.read_u8());
        Ok(BbAddGuildCard {
            guildcard: guildcard,
            name: name,
            team_name: team_name,
            text: text,
            one: one,
            lang: lang,
            section: section,
            char_class: char_class
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbParamHdrReq;
impl_unit_serial!(BbParamHdrReq);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbParamHdr {
    // bb_pkt_hdr_t hdr;
    // struct {
    //     uint32_t size;
    //     uint32_t checksum;
    //     uint32_t offset;
    //     char filename[0x40];
    // } entries[];
    pub params: Vec<ParamHeader>
}
impl Serial for BbParamHdr {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        // There will always be 9 param files. But this is a simplification.
        for p in self.params.iter() {
            try!(p.serialize(dst));
        }
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BbParamChunkReq;
impl_unit_serial!(BbParamChunkReq);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbParamChunk {
    // uint32_t chunk;
    // uint8_t data[];
    pub chunk: u32,
    pub data: Vec<u8>
}
impl Serial for BbParamChunk {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.chunk));
        try!(dst.write_all(&self.data[..]));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbSetFlags(pub u32);
impl Serial for BbSetFlags {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.0));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbSetFlags(try!(src.read_u32::<LE>())))
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbTeamAndKeyData {
    // uint8_t unk[0x114];
    // uint8_t key_config[0x16C];
    // uint8_t joystick_config[0x38];
    // uint32_t guildcard;
    // uint32_t team_id;
    // uint32_t team_info[2];
    // uint16_t team_priv;
    // uint16_t reserved;
    // uint16_t team_name[16];
    // uint8_t team_flag[2048];
    // uint32_t team_rewards[2];
    pub unk: Vec<u8>,
    pub key_config: Vec<u8>,
    pub joy_config: Vec<u8>,
    pub guildcard: u32,
    pub team_id: u32,
    pub team_info: (u32, u32),
    pub team_priv: u32,
    pub team_name: String,
    pub team_rewards: u64
}
impl Serial for BbTeamAndKeyData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.unk, 0x114, dst));
        try!(write_array(&self.key_config, 0x16C, dst));
        try!(write_array(&self.joy_config, 0x38, dst));
        try!(dst.write_u32::<LE>(self.guildcard));
        try!(dst.write_u32::<LE>(self.team_id));
        try!(dst.write_u32::<LE>(self.team_info.0));
        try!(dst.write_u32::<LE>(self.team_info.1));
        try!(dst.write_u32::<LE>(self.team_priv));
        try!(write_utf16_len(&self.team_name, 32, dst));
        try!(dst.write_all(&vec![0u8; 2048][..])); // team flag sooooooooon
        try!(dst.write_u64::<LE>(self.team_rewards));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<BbTeamAndKeyData> {
        let unk = try!(read_array(src, 0x114));
        let key_config = try!(read_array(src, 0x16C));
        let joy_config = try!(read_array(src, 0x38));
        let guildcard = try!(src.read_u32::<LE>());
        let team_id = try!(src.read_u32::<LE>());
        let team_info = (try!(src.read_u32::<LE>()), try!(src.read_u32::<LE>()));
        let team_priv = try!(src.read_u32::<LE>());
        let team_name = try!(read_utf16_len(32, src));
        let team_rewards = try!(src.read_u64::<LE>());
        Ok(BbTeamAndKeyData {
            unk: unk,
            key_config: key_config,
            joy_config: joy_config,
            guildcard: guildcard,
            team_id: team_id,
            team_info: team_info,
            team_priv: team_priv,
            team_name: team_name,
            team_rewards: team_rewards
        })
    }
}

impl Default for BbTeamAndKeyData {
    fn default() -> BbTeamAndKeyData {
        BbTeamAndKeyData {
            // TODO actual BB defaults for these
            unk: vec![0u8; 0x114],
            key_config: default_config::DEFAULT_KEYS.to_vec(),
            joy_config: default_config::DEFAULT_JOY.to_vec(),
            guildcard: 0,
            team_id: 0,
            team_info: (0, 0),
            team_priv: 0,
            team_name: "".to_string(),
            team_rewards: 0xFFFFFFFFFFFFFFFF
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParamHeader {
    // struct {
    //     uint32_t size;
    //     uint32_t checksum;
    //     uint32_t offset;
    //     char filename[0x40];
    // } entries[];
    pub size: u32,
    pub checksum: u32,
    pub offset: u32,
    pub filename: String
}
impl Serial for ParamHeader {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.size));
        try!(dst.write_u32::<LE>(self.checksum));
        try!(dst.write_u32::<LE>(self.offset));
        try!(write_ascii_len(&self.filename, 0x40, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

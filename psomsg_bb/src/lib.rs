//! Message structures for Blue Burst.

#![allow(unused_variables)]

#[macro_use] extern crate psoserial;
extern crate psomsg_common;
#[macro_use] extern crate log;
extern crate byteorder;
extern crate typenum;

use std::io::{Read, Write};
use std::io;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

pub static PSOBB_COPYRIGHT_STRING: &'static [u8] = b"Phantasy Star Online Blue Burst Game Server. Copyright 1999-2004 SONICTEAM.";

pub mod default_config;

use psoserial::Serial;
use psomsg_common::util::read_exact;

pub mod msgs;
pub mod data;
pub mod chara;
pub mod lobby;
pub mod player;
pub mod subcmd;
pub mod game;

pub use self::msgs::*;
pub use psomsg_common::*;
pub use self::data::*;
pub use self::chara::*;
pub use self::lobby::*;
pub use self::player::*;
pub use self::subcmd::*;
pub use self::game::*;

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
                if buf.len() % 8 != 0 {
                    let buf_len = buf.len();
                    debug!("contents need to be padded by {}", 8 - buf_len % 8);
                    buf.append(&mut vec![0u8; (8 - buf_len % 8) as usize]);
                    // size += (8 - buf_len % 8) as u16;
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
                debug!("Serializing into: size {}, msg_type 0x{:x}, flags {}", size + 8, msg_type, flags);
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
                try!(read_exact(src, &mut hdr_buf[..]));
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
                debug!("size: {size}, type: 0x{msg_type:x}, flags: {flags}", size=size, msg_type=msg_type, flags=flags);

                let padding = if size % 8 == 0 { 0 } else { 8 - (size % 8) };

                let mut msg_buf = vec![0u8; (size + padding) as usize - 8];
                if size > 8 {
                    try!(read_exact(src, &mut msg_buf));
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

        $(
            impl From<$name> for Message {
                #[inline(always)]
                fn from(val: $name) -> Message {
                    Message::$name(0, val)
                }
            }

            impl From<(u32, $name)> for Message {
                #[inline(always)]
                fn from(val: (u32, $name)) -> Message {
                    Message::$name(val.0, val.1)
                }
            }
        )*

        impl From<(u16, u32, Vec<u8>)> for Message {
            #[inline(always)]
            fn from(val: (u16, u32, Vec<u8>)) -> Message {
                Message::Unknown(val.0, val.1, val.2)
            }
        }
    }
}

gen_message_enum! {
    0x0001 => BbMsg1,
    0x0003 => BbWelcome,
    0x0005 => Goodbye,
    0x0006 => BbChat,
    0x0007 => BlockList,
    0x0008 => BbGameList,
    0x0010 => MenuSelect,
    0x0011 => BbInfoReply,
    0x0019 => Redirect,
    0x0060 => BbSubCmd60,
    0x0061 => BbCharDat,
    0x0062 => BbSubCmd62,
    0x0064 => BbGameJoin,
    0x0065 => BbGameAddMember,
    0x0066 => BbGameLeave,
    0x0067 => LobbyJoin,
    0x0068 => LobbyAddMember,
    0x0069 => LobbyLeave,
    0x006C => BbSubCmd6C,
    0x006D => BbSubCmd6D,
    0x006F => DoneBursting,
    0x0083 => LobbyList,
    0x0084 => LobbyChange,
    0x0088 => LobbyArrowList,
    0x008A => BbGameName,
    0x0093 => BbLogin,
    0x0095 => CharDataRequest,
    0x0098 => BbPlayerLeaveGame,
    0x001A => LargeMsg,
    0x001D => Ping,
    0x00A0 => ShipList,
    0x00B1 => Timestamp,
    0x00C1 => BbCreateGame,
    0x01DC => BbGuildCardHdr,
    0x02DC => BbGuildCardChunk,
    0x03DC => BbGuildCardChunkReq,
    0x00E0 => BbOptionRequest,
    0x00E2 => BbOptionConfig,
    0x00E3 => BbCharSelect,
    0x00E4 => BbCharAck,
    0x00E5 => BbCharInfo,
    0x00E6 => BbSecurity,
    0x00E7 => BbFullChar,
    0x01E8 => BbChecksum,
    0x02E8 => BbChecksumAck,
    0x03E8 => BbGuildRequest,
    0x04E8 => BbAddGuildCard,
    0x15EA => BbTeamInfo,
    0x01EB => BbParamHdr,
    0x02EB => BbParamChunk,
    0x03EB => BbParamChunkReq,
    0x04EB => BbParamHdrReq,
    0x00EC => BbSetFlags,
    0x00EE => BbScrollMsg
}

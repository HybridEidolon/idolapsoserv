//! Message structures for Blue Burst.

use std::io::{Read, Write};
use std::io;

use byteorder::{LittleEndian as LE, BigEndian as BE, ReadBytesExt, WriteBytesExt};

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
                use std::borrow::BorrowMut;
                use std::io::Cursor;
                let mut buf = Vec::with_capacity(512);
                let msg_type: u16;
                let size: u16;
                let flags: u32;
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
                try!(dst.write_u16::<LE>(size + 8));
                try!(dst.write_u16::<LE>(msg_type));
                try!(dst.write_u32::<LE>(flags));
                let (buf_slice, _) = buf.split_at(size as usize);
                try!(dst.write_all(buf_slice));

                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<Self> {
                // parse header
                let size = try!(src.read_u16::<LE>());
                let msg_type = try!(src.read_u16::<LE>());
                let flags = try!(src.read_u32::<LE>());
                match msg_type {
                    $($id => Ok(Message::$name(flags, try!($name::deserialize(src)))),)*
                    a => {
                        use std::borrow::BorrowMut;
                        let mut buf = vec![0; size as usize - 8];
                        {
                            if try!(src.read(buf.borrow_mut())) != size as usize - 8 {
                                return Err(io::Error::new(io::ErrorKind::Other,
                                    format!("expected {} bytes", size - 8)))
                            }
                        }
                        Ok(Message::Unknown(a, flags, buf))
                    }
                }
            }
        }
    }
}

gen_message_enum! {
    0x03 => Welcome
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Welcome(pub Vec<u8>, pub Vec<u8>);
impl Serial for Welcome {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_all(b"Phantasy Star Online Blue Burst Game Server. Copyright 1999-2004 SONICTEAM."));
        try!(dst.write_all(&self.0[..]));
        try!(dst.write_all(&self.1[..]));
        Ok(())
    }
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

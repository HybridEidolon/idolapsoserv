//! Trait for serializing types into `Read`/`Write` streams.

extern crate byteorder;

use std::io;
use std::io::{Read, Write};
use std::net::Ipv4Addr;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

#[macro_export]
/// Define a struct and derive `Serial`, `Debug`, and `Clone` for it.
macro_rules! derive_serial {
    ($name:ident {$(pub $fname:ident: $fty:ty),+}) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $fname: $fty),*
        }

        // because typenum doesn't implement it...
        impl Clone for $name {
            fn clone(&self) -> Self {
                $name {
                    $($fname: self.$fname.clone()),*
                }
            }
        }

        impl Serial for $name {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                $(try!(self.$fname.serialize(dst));)*
                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<Self> {
                $(let $fname = try!(<$fty as Serial>::deserialize(src));)*
                Ok($name {
                    $($fname: $fname),*
                })
            }
        }
    };
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name;
        impl Serial for $name {
            fn serialize(&self, _: &mut Write) -> io::Result<()> {
                Ok(())
            }
            fn deserialize(_: &mut Read) -> io::Result<Self> {
                Ok($name)
            }
        }
    }
}

macro_rules! prim_num_serial {
    ($name:ident, $read_func:ident, $write_func:ident) => {
        impl Serial for $name {
            #[inline(always)]
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                try!(dst.$write_func::<LE>(*self));
                Ok(())
            }

            #[inline(always)]
            fn deserialize(src: &mut Read) -> io::Result<Self> {
                let v = try!(src.$read_func::<LE>());
                Ok(v)
            }
        }
    }
}

prim_num_serial!(u16, read_u16, write_u16);
prim_num_serial!(u32, read_u32, write_u32);
prim_num_serial!(u64, read_u64, write_u64);
prim_num_serial!(i16, read_i16, write_i16);
prim_num_serial!(i32, read_i32, write_i32);
prim_num_serial!(i64, read_i64, write_i64);
prim_num_serial!(f32, read_f32, write_f32);

impl Serial for u8 {
    #[inline(always)]
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u8(*self));
        Ok(())
    }

    #[inline(always)]
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let v = try!(src.read_u8());
        Ok(v)
    }
}

impl Serial for i8 {
    #[inline(always)]
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_i8(*self));
        Ok(())
    }

    #[inline(always)]
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let v = try!(src.read_i8());
        Ok(v)
    }
}

/// A type that can be serialized and deserialized into a `Read` or `Write`
/// trait object.
pub trait Serial: Sized {
    /// Write the full contents of this type into `dst`. Will return `Err` if
    /// the entire type is not serialized for any reason.
    fn serialize(&self, dst: &mut Write) -> io::Result<()>;
    /// Read a complete `Self` from `src`. Will return `Err` if the entire type
    /// is not deserialized for any reason.
    fn deserialize(src: &mut Read) -> io::Result<Self>;
}

// Other Serial implementations
impl Serial for Ipv4Addr {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        let octets = self.octets();
        try!(dst.write_all(&octets[..]));
        Ok(())
    }

    fn deserialize(_: &mut Read) -> io::Result<Self> {
        unimplemented!()
    }
}

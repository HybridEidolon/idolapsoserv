//! Trait for serializing types into `Read`/`Write` streams.

extern crate byteorder;
#[macro_use] extern crate log;
extern crate encoding;

use std::io;
use std::io::{Read, Write};
use std::net::Ipv4Addr;

pub mod util;

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
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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

#[macro_export]
/// Same as `derive_serial` but also deriving Default.
macro_rules! derive_serial_default {
    ($name:ident {$(pub $fname:ident: $fty:ty),+}) => {
        #[derive(Debug, Default)]
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
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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

macro_rules! impl_serial_array {
    ($size:expr) => {
        impl<T: Serial> Serial for [T; $size] {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                for i in 0..$size {
                    try!(self[i].serialize(dst));
                }
                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<[T; $size]> {
                use std::mem::uninitialized;
                use std::mem::swap;
                use std::mem::forget;
                let mut ret = unsafe { uninitialized::<[T; $size]>() };
                for i in 0..$size {
                    let mut uninit = try!(Serial::deserialize(src));
                    swap(&mut uninit, &mut ret[i]);
                    forget(uninit);
                }
                Ok(ret)
            }
        }
    }
}

// TYPE. LEVEL. NUMBERS. PLS.
impl_serial_array!(1);
impl_serial_array!(2);
impl_serial_array!(3);
impl_serial_array!(4);
impl_serial_array!(5);
impl_serial_array!(6);
impl_serial_array!(7);
impl_serial_array!(8);
impl_serial_array!(9);
impl_serial_array!(10);
impl_serial_array!(11);
impl_serial_array!(12);
impl_serial_array!(13);
impl_serial_array!(14);
impl_serial_array!(15);
impl_serial_array!(16);
impl_serial_array!(17);
impl_serial_array!(18);
impl_serial_array!(19);
impl_serial_array!(20);
impl_serial_array!(21);
impl_serial_array!(22);
impl_serial_array!(23);
impl_serial_array!(24);
impl_serial_array!(25);
impl_serial_array!(26);
impl_serial_array!(27);
impl_serial_array!(28);
impl_serial_array!(29);
impl_serial_array!(30);
impl_serial_array!(31);
impl_serial_array!(32);


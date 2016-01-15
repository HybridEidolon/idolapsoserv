use std::io;
use std::io::{Write, Read, Cursor};

use psoserial::Serial;
use psomsg_common::util::*;

use super::*;

macro_rules! impl_subcmd_enum {
    ($numname:ident = $($id:expr => $name:ident),*) => {
        #[derive(Clone, Debug)]
        pub enum $numname {
            Unknown {
                cmd: u8,
                client_id: u8,
                unused: u8,
                data: Vec<u8>
            },
            $($name {
                client_id: u8,
                unused: u8,
                data: $name
            }),*
        }

        impl Serial for $numname {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                let w_cmd;
                let w_size: u8;
                let w_client_id;
                let w_unused;
                let w_data;
                match self {
                    &$numname::Unknown { cmd, client_id, unused, ref data } => {
                        w_cmd = cmd;
                        w_client_id = client_id;
                        w_unused = unused;
                        w_data = data.clone();
                        if w_data.len() % 4 != 0 {
                            warn!("Unknown subcommand is not divided on 4 byte boundary, not sizeable. Continuing anyway.");
                        }
                        w_size = (w_data.len() / 4) as u8 + 1;
                    },
                    $(&$numname::$name { client_id, unused, ref data } => {
                        w_cmd = $id;
                        w_client_id = client_id;
                        w_unused = unused;
                        let mut cursor = Cursor::new(Vec::new());
                        try!(data.serialize(&mut cursor));
                        w_data = cursor.into_inner();
                        if w_data.len() % 4 != 0 {
                            warn!("Subcommand {} size is not divided on 4 byte boundary, not sizeable. Continuing anyway.", stringify!($name));
                        }
                        w_size = (w_data.len() / 4) as u8 + 1;
                    }),*
                }

                try!(w_cmd.serialize(dst));
                try!(w_size.serialize(dst));
                try!(w_client_id.serialize(dst));
                try!(w_unused.serialize(dst));
                try!(dst.write_all(&w_data));
                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<$numname> {
                let cmd: u8 = try!(Serial::deserialize(src));
                let size: u8 = try!(Serial::deserialize(src));
                let client_id: u8 = try!(Serial::deserialize(src));
                let unused: u8 = try!(Serial::deserialize(src));
                debug!("{} header: cmd=0x{:x}, size=0x{:x}, client_id=0x{:x}, unused=0x{:x}", stringify!($numname), cmd, size, client_id, unused);
                let ret = match cmd {
                    $($id => {
                        let data = try!($name::deserialize(src));
                        $numname::$name {
                            client_id: client_id,
                            unused: unused,
                            data: data
                        }
                    },)*
                    c => {
                        if size == 0 {
                            $numname::Unknown {
                                cmd: cmd,
                                client_id: client_id,
                                unused: unused,
                                data: Vec::new()
                            }
                        } else {
                            let mut buf = vec![0u8; (size as usize - 1) * 4];
                            try!(read_exact(src, &mut buf));
                            $numname::Unknown {
                                cmd: cmd,
                                client_id: client_id,
                                unused: unused,
                                data: buf
                            }
                        }
                    }
                };
                Ok(ret)
            }
        }
    }
}

macro_rules! impl_subcmd_6d_enum {
    ($numname:ident = $($id:expr => $name:ident),*) => {
        #[derive(Clone, Debug)]
        pub enum $numname {
            Unknown {
                cmd: u8,
                flags: u8,
                unused: u16,
                data: Vec<u8>
            },
            $($name {
                flags: u8,
                unused: u16,
                data: $name
            }),*
        }

        impl Serial for $numname {
            fn serialize(&self, dst: &mut Write) -> io::Result<()> {
                let w_cmd: u8;
                let w_flags: u8;
                let w_unused: u16;
                let w_size: u32;
                let w_data;
                match self {
                    &$numname::Unknown { cmd, flags, unused, ref data } => {
                        w_cmd = cmd;
                        w_flags = flags;
                        w_unused = unused;
                        w_data = data.clone();
                        w_size = (w_data.len() + 8) as u32;
                    },
                    $(&$numname::$name { flags, unused, ref data } => {
                        w_cmd = $id;
                        w_flags = flags;
                        w_unused = unused;
                        let mut cursor = Cursor::new(Vec::new());
                        try!(data.serialize(&mut cursor));
                        w_data = cursor.into_inner();
                        w_size = (w_data.len() + 8) as u32;
                    }),*
                }

                try!(w_cmd.serialize(dst));
                try!(w_flags.serialize(dst));
                try!(w_unused.serialize(dst));
                try!(w_size.serialize(dst));
                try!(dst.write_all(&w_data));
                Ok(())
            }

            fn deserialize(src: &mut Read) -> io::Result<$numname> {
                let cmd: u8 = try!(Serial::deserialize(src));
                let flags: u8 = try!(Serial::deserialize(src));
                let unused: u16 = try!(Serial::deserialize(src));
                let size: u32 = try!(Serial::deserialize(src));
                debug!("{} header: cmd=0x{:x}, flags=0x{:x}, unused=0x{:x}, size=0x{:x}", stringify!($numname), cmd, flags, unused, size);
                let ret = match cmd {
                    $($id => {
                        let data = try!($name::deserialize(src));
                        $numname::$name {
                            flags: flags,
                            unused: unused,
                            data: data
                        }
                    },)*
                    c => {
                        if size == 0 {
                            $numname::Unknown {
                                cmd: cmd,
                                flags: flags,
                                unused: unused,
                                data: Vec::new()
                            }
                        } else {
                            let mut buf = vec![0u8; size as usize - 8];
                            try!(read_exact(src, &mut buf));
                            $numname::Unknown {
                                cmd: cmd,
                                flags: flags,
                                unused: unused,
                                data: buf
                            }
                        }
                    }
                };
                Ok(ret)
            }
        }
    }
}

impl_subcmd_enum! { BbSubCmd60 =
    0x30 => Bb60LevelUp,
    0x2A => Bb60DropItem,
    0x63 => Bb60DestroyItem,
    0x72 => Bb60DoneBurst,
    0x6F => QuestData1,
    0xBF => Bb60GiveExp,
    0xC8 => Bb60ReqExp
}

impl_subcmd_enum! { BbSubCmd6C =
    0x30 => Bb60LevelUp,
    0x2A => Bb60DropItem,
    0x63 => Bb60DestroyItem,
    0x72 => Bb60DoneBurst,
    0x6F => QuestData1,
    0xBF => Bb60GiveExp,
    0xC8 => Bb60ReqExp
}

impl_subcmd_enum! { BbSubCmd62 =
    0x60 => Bb62ItemReq,
    0xB5 => Bb62ShopReq,
    0xB6 => Bb62ShopInv,
    0xBB => Bb62OpenBank
}

impl_subcmd_6d_enum! { BbSubCmd6D =
    0xBC => Bb6DBankInv
}

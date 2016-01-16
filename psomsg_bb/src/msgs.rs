use psoserial::Serial;

use std::io::{Read, Write};
use std::io;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

use psomsg_common::util::*;
use super::PSOBB_COPYRIGHT_STRING;
use super::data::*;
use super::chara::*;
use super::player::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbWelcome(pub Vec<u8>, pub Vec<u8>);
impl Serial for BbWelcome {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        let padding = vec![0u8; 0x60 - PSOBB_COPYRIGHT_STRING.len()];
        assert_eq!(0x60, PSOBB_COPYRIGHT_STRING.len() + padding.len());
        try!(dst.write_all(PSOBB_COPYRIGHT_STRING));
        try!(dst.write_all(&padding[..]));
        try!(write_array(&self.0, 48, dst));
        try!(write_array(&self.1, 48, dst));
        Ok(())
    }
    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(read_array::<u8>(0x60, src));
        let server_key = try!(read_array(48, src));
        let client_key = try!(read_array(48, src));
        Ok(BbWelcome(server_key, client_key))
    }
}

impl Default for BbWelcome {
    fn default() -> BbWelcome {
        BbWelcome(vec![0; 48], vec![0; 48])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbLogin {
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
    pub security_data: BbSecurityData
}
impl Serial for BbLogin {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.tag.serialize(dst));
        try!(self.guildcard.serialize(dst));
        try!(self.version.serialize(dst));
        try!(write_array(&self.unk, 6, dst));
        try!(self.team_id.serialize(dst));
        try!(write_ascii_len(&self.username, 16, dst));
        try!(dst.write_all(&[0u8; 32]));
        try!(write_ascii_len(&self.password, 16, dst));
        try!(dst.write_all(&[0u8; 40]));
        try!(write_array(&self.hw_info, 8, dst));
        try!(self.security_data.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let tag = try!(src.read_u32::<LE>());
        let guildcard = try!(src.read_u32::<LE>());
        let version = try!(src.read_u16::<LE>());
        let unk = try!(read_array(6, src));
        let team_id = try!(src.read_u32::<LE>());
        let username = try!(read_ascii_len(16, src));
        try!(src.read(&mut [0u8; 32]));
        let password = try!(read_ascii_len(16, src));
        try!(src.read(&mut [0u8; 40]));
        let hw_info = try!(read_array(8, src));
        let security_data = try!(BbSecurityData::deserialize(src));
        Ok(BbLogin {
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

impl Default for BbLogin {
    fn default() -> BbLogin {
        BbLogin {
            tag: 0,
            guildcard: 0,
            version: 0,
            unk: vec![0; 6],
            team_id: 0,
            username: Default::default(),
            password: Default::default(),
            hw_info: vec![0; 8],
            security_data: Default::default()
        }
    }
}

derive_serial!(BbOptionRequest);

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
        try!(self.slot.serialize(dst));
        try!(if self.selecting { 1u32 } else { 0u32 }.serialize(dst));
        Ok(())
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
        try!(self.0.serialize(dst));
        try!(self.1.serialize(dst));
        try!(if self.2 { 1u32 } else { 0u32 }.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let unk = try!(src.read_u32::<LE>());
        let chunk = try!(src.read_u32::<LE>());
        let cont = match try!(src.read_u32::<LE>()) { 0 => false, _ => true };
        Ok(BbGuildCardChunkReq(unk, chunk, cont))
    }
}

derive_serial!(BbGuildRequest);

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
        let unk = try!(Serial::deserialize(src));
        let chunk = try!(Serial::deserialize(src));
        let mut data = Vec::new(); try!(src.read_to_end(&mut data)); // TODO this is dangerous
        Ok(BbGuildCardChunk {
            unk: unk,
            chunk: chunk,
            data: data
        })
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
        try!(self.guildcard.serialize(dst));
        try!(write_utf16_len(&self.name, 48, dst));
        try!(write_utf16_len(&self.team_name, 32, dst));
        try!(write_utf16_len(&self.text, 88*2, dst));
        try!(self.one.serialize(dst));
        try!(self.lang.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.char_class.serialize(dst));
        Ok(())
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

derive_serial!(BbParamHdrReq);

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
        let mut params = Vec::new();
        loop {
            // very unsafe...
            let paramhdr = match ParamHeader::deserialize(src) {
                Ok(p) => p,
                Err(_) => break
            };
            params.push(paramhdr);
        }
        Ok(BbParamHdr {
            params: params
        })
    }
}

derive_serial!(BbParamChunkReq);

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
        let chunk = try!(Serial::deserialize(src));
        let mut data = Vec::new(); try!(src.read_to_end(&mut data));
        data.truncate(0x6800); // max param file size
        Ok(BbParamChunk {
            chunk: chunk,
            data: data
        })
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
        let err_code = try!(Serial::deserialize(src));
        let tag = try!(Serial::deserialize(src));
        let guildcard = try!(Serial::deserialize(src));
        let team_id = try!(Serial::deserialize(src));
        let security_data = try!(Serial::deserialize(src));
        let caps = try!(Serial::deserialize(src));
        Ok(BbSecurity {
            err_code: err_code,
            tag: tag,
            guildcard: guildcard,
            team_id: team_id,
            security_data: security_data,
            caps: caps
        })
    }
}

#[derive(Clone, Debug)]
pub struct BbCharInfo(pub u32, pub BbMiniCharData);
impl Serial for BbCharInfo {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        try!(self.1.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbCharInfo(try!(u32::deserialize(src)), try!(BbMiniCharData::deserialize(src))))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbFullChar(pub BbFullCharData);
impl Serial for BbFullChar {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbFullChar(try!(Serial::deserialize(src))))
    }
}

#[derive(Clone, Debug)]
pub struct BbCharDat(pub BbPlayerData);
impl Serial for BbCharDat {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbCharDat(try!(Serial::deserialize(src))))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbInfoReply(pub String);
impl Serial for BbInfoReply {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(0u64.serialize(dst));
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(u64::deserialize(src));
        let msg = try!(read_utf16(src));
        Ok(BbInfoReply(msg))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbScrollMsg(pub String);
impl Serial for BbScrollMsg {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(0u64.serialize(dst));
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(u64::deserialize(src));
        let msg = try!(read_utf16(src));
        Ok(BbScrollMsg(msg))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbSecurityData {
    // uint32_t magic;                     /* Must be 0xDEADBEEF */
    // uint8_t slot;                       /* Selected character */
    // uint8_t sel_char;                   /* Have they selected a character? */
    // uint8_t reserved[34];               /* Set to 0 */
    pub magic: u32,
    pub slot: u8,
    pub sel_char: u8,
    pub reserved: Vec<u8>
}

impl Serial for BbSecurityData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(dst.write_u32::<LE>(self.magic));
        try!(dst.write_u8(self.slot));
        try!(dst.write_u8(self.sel_char));
        try!(write_array(&self.reserved, 34, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let magic = try!(src.read_u32::<LE>());
        let slot = try!(src.read_u8());
        let sel_char = try!(src.read_u8());
        let reserved = try!(read_array(34, src));
        Ok(BbSecurityData {
            magic: magic,
            slot: slot,
            sel_char: sel_char,
            reserved: reserved
        })
    }
}

impl Default for BbSecurityData {
    fn default() -> BbSecurityData {
        BbSecurityData {
            magic: 0,
            slot: 0,
            sel_char: 0,
            reserved: vec![0; 34]
        }
    }
}

#[derive(Clone, Debug)]
pub struct BbChat(pub u32, pub String);
impl Serial for BbChat {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(0u32.serialize(dst));
        try!(self.0.serialize(dst));
        try!(write_utf16(&self.1, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(u32::deserialize(src));
        let gc = try!(Serial::deserialize(src));
        let text = try!(read_utf16(src));
        Ok(BbChat(gc, text))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BbUpdateOptions(pub u32);
impl Serial for BbUpdateOptions {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.0.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbUpdateOptions(try!(Serial::deserialize(src))))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbUpdateKeys(pub Vec<u8>);
impl Serial for BbUpdateKeys {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.0, 364, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbUpdateKeys(try!(read_array(364, src))))
    }
}

#[derive(Clone, Debug, Default)]
pub struct BbUpdateJoy(pub Vec<u8>);
impl Serial for BbUpdateJoy {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.0, 56, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        Ok(BbUpdateJoy(try!(read_array(56, src))))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BbTeamInfo {
    pub guildcard: u32,
    pub team_id: u32,
    pub reserved: Vec<u8>,
    pub priv_level: u32,
    pub team_name: String,
    pub guildcard2: u32,
    pub client_id: u32,
    pub name: String,
    pub reserved2: Vec<u8>,
    pub team_flag: Vec<u8>
}
impl Serial for BbTeamInfo {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.guildcard.serialize(dst)); // 0x0
        try!(self.team_id.serialize(dst)); // 0x4
        try!(write_array(&self.reserved, 12, dst)); // 0x8
        try!(self.priv_level.serialize(dst)); // 0x14
        try!(write_utf16_len(&self.team_name, 24, dst)); // 0x18
        try!(0x00986C84u32.serialize(dst));
        try!(self.guildcard2.serialize(dst));
        try!(self.client_id.serialize(dst)); //
        try!(write_utf16_len(&self.name, 24, dst)); // 0x44
        try!(write_array(&self.reserved2, 8, dst));
        try!(write_array(&self.team_flag, 0x800, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<BbTeamInfo> {
        let guildcard = try!(Serial::deserialize(src));
        let team_id = try!(Serial::deserialize(src));
        let reserved = try!(read_array(12, src));
        let priv_level = try!(Serial::deserialize(src));
        let team_name = try!(read_utf16_len(24, src));
        let _: u32 = try!(Serial::deserialize(src));
        let guildcard2 = try!(Serial::deserialize(src));
        let client_id = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(24, src));
        let reserved2 = try!(read_array(8, src));
        let team_flag = try!(read_array(2048, src));
        Ok(BbTeamInfo {
            guildcard: guildcard,
            team_id: team_id,
            reserved: reserved,
            priv_level: priv_level,
            team_name: team_name,
            guildcard2: guildcard2,
            client_id: client_id,
            name: name,
            reserved2: reserved2,
            team_flag: team_flag
        })
    }
}

impl Default for BbTeamInfo {
    fn default() -> BbTeamInfo {
        BbTeamInfo {
            guildcard: 0,
            team_id: 0,
            reserved: vec![0; 12],
            priv_level: 0,
            team_name: "".to_string(),
            guildcard2: 0,
            client_id: 0,
            name: "".to_string(),
            reserved2: vec![0; 8],
            team_flag: vec![0u8; 0x800]
        }
    }
}

#[derive(Clone, Debug)]
pub struct BbMsg1(pub String);
impl Serial for BbMsg1 {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(0u32.serialize(dst));
        try!(0u32.serialize(dst));
        try!(write_utf16(&self.0, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        try!(u32::deserialize(src));
        try!(u32::deserialize(src));
        let msg = try!(read_utf16(src));
        Ok(BbMsg1(msg))
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
        let size = try!(Serial::deserialize(src));
        let checksum = try!(Serial::deserialize(src));
        let offset = try!(Serial::deserialize(src));
        let filename = try!(read_ascii_len(0x40, src));
        Ok(ParamHeader {
            size: size,
            checksum: checksum,
            offset: offset,
            filename: filename
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use psoserial::Serial;
    use super::*;
    use super::super::Message;

    #[test]
    fn test_bb_full_char_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a: Message = BbFullChar::default().into();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 0x39A0);
        let array = cursor.into_inner();
    }

    #[test]
    fn test_bb_key_team_config_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = BbTeamAndKeyData::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 0xAF0);
    }

    #[test]
    fn test_bb_option_config() {
        let mut cursor = Cursor::new(Vec::new());
        let a: Message = BbOptionConfig(BbTeamAndKeyData::default()).into();
        a.serialize(&mut cursor).unwrap();
        let position = cursor.position();
        let array = cursor.into_inner();
        assert_eq!(position, 2808);
    }

    #[test]
    fn test_security_data_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = BbSecurityData::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 40);
    }

    #[test]
    fn test_bb_login_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a: Message = BbLogin::default().into();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 184);
        let array = cursor.into_inner();
        println!("Check header's size");
        assert_eq!(array[0], 180);
        assert_eq!(array[1], 0);
    }

    #[test]
    fn test_welcome_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a: Message = BbWelcome::default().into();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 200);
        let array = cursor.into_inner();
    }
}

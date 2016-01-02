use std::io;
use std::io::{Read, Write};

use psomsg_common::util::*;
use psoserial::Serial;
use super::chara::*;

#[derive(Clone, Debug)]
pub struct PlayerHdr {
    // uint32_t tag;
    // uint32_t guildcard;
    // uint32_t unk1[5];
    // uint32_t client_id;
    // uint16_t name[16];
    // uint32_t unk2;
    pub tag: u32,
    pub guildcard: u32,
    pub unk1: Vec<u32>,
    pub client_id: u32,
    pub name: String,
    pub unk2: u32
}
impl Serial for PlayerHdr {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.tag.serialize(dst));
        try!(self.guildcard.serialize(dst));
        try!(write_array(&self.unk1, 5, dst));
        try!(self.client_id.serialize(dst));
        try!(write_utf16_len(&self.name, 32, dst));
        try!(self.unk2.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let tag = try!(Serial::deserialize(src));
        let guildcard = try!(Serial::deserialize(src));
        let unk1 = try!(read_array(5, src));
        let client_id = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(32, src));
        let unk2 = try!(Serial::deserialize(src));
        Ok(PlayerHdr {
            tag: tag,
            guildcard: guildcard,
            unk1: unk1,
            client_id: client_id,
            name: name,
            unk2: unk2
        })
    }
}
impl Default for PlayerHdr {
    fn default() -> Self {
        PlayerHdr {
            tag: Default::default(),
            guildcard: Default::default(),
            unk1: vec![Default::default(); 5],
            client_id: Default::default(),
            name: Default::default(),
            unk2: Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct BbPlayerData {
    // sylverant_inventory_t inv;
    // sylverant_bb_char_t character;
    // uint8_t c_rank[0x0174];
    // uint16_t infoboard[172];
    // uint32_t blacklist[30];
    // uint32_t autoreply_enabled;
    // uint16_t autoreply[];
    pub inventory: Inventory,
    pub chara: BbChar,
    pub c_rank: Vec<u8>,
    pub infoboard: String,
    pub blacklist: Vec<u32>,
    pub autoreply_enabled: bool,
    pub autoreply: String
}
impl Serial for BbPlayerData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.inventory.serialize(dst));
        try!(self.chara.serialize(dst));
        try!(write_array(&self.c_rank, 0x174, dst));
        try!(write_utf16_len(&self.infoboard, 172*2, dst));
        try!(write_array(&self.blacklist, 30, dst));
        try!(u32::serialize(&if self.autoreply_enabled { 1u32 } else { 0u32 }, dst));
        try!(write_utf16(&self.autoreply, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let inventory = try!(Serial::deserialize(src));
        let chara = try!(Serial::deserialize(src));
        let c_rank = try!(read_array(0x174, src));
        let infoboard = try!(read_utf16_len(172*2, src));
        let blacklist = try!(read_array(30, src));
        let autoreply_enabled = if try!(u32::deserialize(src)) == 0 { false } else { true };
        let autoreply = try!(read_utf16(src));
        Ok(BbPlayerData {
            inventory: inventory,
            chara: chara,
            c_rank: c_rank,
            infoboard: infoboard,
            blacklist: blacklist,
            autoreply_enabled: autoreply_enabled,
            autoreply: autoreply
        })
    }
}
impl Default for BbPlayerData {
    fn default() -> Self {
        BbPlayerData {
            inventory: Default::default(),
            chara: Default::default(),
            c_rank: vec![Default::default(); 0x174],
            infoboard: Default::default(),
            blacklist: vec![Default::default(); 30],
            autoreply_enabled: Default::default(),
            autoreply: Default::default()
        }
    }
}

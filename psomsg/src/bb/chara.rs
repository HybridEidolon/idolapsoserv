use ::Serial;
use ::util::*;

use super::BbTeamAndKeyData;

use std::io;
use std::io::{Read, Write};

#[derive(Clone, Debug, Default)]
pub struct InvItem {
    pub equipped: u16,
    pub tech: u16,
    pub flags: u32,
    pub item_data: ItemData
}
impl Serial for InvItem {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.equipped.serialize(dst));
        try!(self.tech.serialize(dst));
        try!(self.flags.serialize(dst));
        try!(self.item_data.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let equipped = try!(u16::deserialize(src));
        let tech = try!(u16::deserialize(src));
        let flags = try!(u32::deserialize(src));
        let item_data = try!(ItemData::deserialize(src));
        Ok(InvItem {
            equipped: equipped,
            tech: tech,
            flags: flags,
            item_data: item_data
        })
    }
}

#[derive(Clone, Debug)]
pub struct ItemData {
    pub data: Vec<u8>,
    pub item_id: u32,
    pub data2: Vec<u8>
}
impl Serial for ItemData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.data, 12, dst));
        try!(self.item_id.serialize(dst));
        try!(write_array(&self.data2, 4, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(read_array(12, src));
        let item_id = try!(u32::deserialize(src));
        let data2 = try!(read_array(4, src));
        Ok(ItemData {
            data: data,
            item_id: item_id,
            data2: data2
        })
    }
}
impl Default for ItemData {
    fn default() -> Self {
        ItemData {
            data: vec![0; 12],
            item_id: Default::default(),
            data2: vec![0; 4]
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub item_count: u8,
    pub hp_mats: u8,
    pub tp_mats: u8,
    pub lang: u8,
    pub items: Vec<InvItem>
}
impl Serial for Inventory {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.item_count.serialize(dst));
        try!(self.hp_mats.serialize(dst));
        try!(self.tp_mats.serialize(dst));
        try!(self.lang.serialize(dst));
        try!(write_array(&self.items, 30, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let item_count = try!(u8::deserialize(src));
        let hp_mats = try!(u8::deserialize(src));
        let tp_mats = try!(u8::deserialize(src));
        let lang = try!(u8::deserialize(src));
        let items = try!(read_array(30, src));
        Ok(Inventory {
            item_count: item_count,
            hp_mats: hp_mats,
            tp_mats: tp_mats,
            lang: lang,
            items: items
        })
    }
}
impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            item_count: 0,
            hp_mats: 0,
            tp_mats: 0,
            lang: 0,
            items: vec![InvItem::default(); 30]
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemBank {
    pub item_count: u32,
    pub meseta: u32,
    pub items: Vec<ItemData>
}
impl Serial for ItemBank {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.item_count.serialize(dst));
        try!(self.meseta.serialize(dst));
        try!(write_array(&self.items, 200, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let item_count = try!(u32::deserialize(src));
        let meseta = try!(u32::deserialize(src));
        let items = try!(read_array(200, src));
        Ok(ItemBank {
            item_count: item_count,
            meseta: meseta,
            items: items
        })
    }
}
impl Default for ItemBank {
    fn default() -> Self {
        ItemBank {
            item_count: 0,
            meseta: 0,
            items: vec![ItemData::default(); 200]
        }
    }
}

#[derive(Clone, Debug)]
pub struct BbChar {
    pub atp: u16,
    pub mst: u16,
    pub evp: u16,
    pub hp: u16,
    pub dfp: u16,
    pub ata: u16,
    pub lck: u16,
    pub unk1: u16,
    pub unk2_1: u32,
    pub unk2_2: u32,
    pub level: u32,
    pub exp: u32,
    pub meseta: u32,
    pub guildcard: String,
    pub unk3: u64,
    pub name_color: u32,
    pub model: u8,
    pub name_color_checksum: u32,
    pub section: u8,
    pub class: u8,
    pub v2flags: u8,
    pub version: u8,
    pub v1flags: u32,
    pub costume: u16,
    pub skin: u16,
    pub face: u16,
    pub head: u16,
    pub hair: u16,
    pub hair_r: u16,
    pub hair_g: u16,
    pub hair_b: u16,
    pub prop_x: f32,
    pub prop_y: f32,
    pub name: String,
    pub config: Vec<u8>,
    pub techniques: Vec<u8>
}
impl Serial for BbChar {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.atp.serialize(dst));
        try!(self.mst.serialize(dst));
        try!(self.evp.serialize(dst));
        try!(self.hp.serialize(dst));
        try!(self.dfp.serialize(dst));
        try!(self.ata.serialize(dst));
        try!(self.lck.serialize(dst));
        try!(self.unk1.serialize(dst));
        try!(self.unk2_1.serialize(dst));
        try!(self.unk2_2.serialize(dst));
        try!(self.level.serialize(dst));
        try!(self.exp.serialize(dst));
        try!(self.meseta.serialize(dst));
        try!(write_ascii_len(&self.guildcard, 16, dst));
        try!(self.unk3.serialize(dst));
        try!(self.name_color.serialize(dst));
        try!(self.model.serialize(dst));
        try!(dst.write_all(&[0; 15]));
        try!(self.name_color_checksum.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.class.serialize(dst));
        try!(self.v2flags.serialize(dst));
        try!(self.version.serialize(dst));
        try!(self.v1flags.serialize(dst));
        try!(self.costume.serialize(dst));
        try!(self.skin.serialize(dst));
        try!(self.face.serialize(dst));
        try!(self.head.serialize(dst));
        try!(self.hair.serialize(dst));
        try!(self.hair_r.serialize(dst));
        try!(self.hair_g.serialize(dst));
        try!(self.hair_b.serialize(dst));
        try!(self.prop_x.serialize(dst));
        try!(self.prop_y.serialize(dst));
        try!(write_utf16_len(&self.name, 32, dst));
        try!(write_array(&self.config, 0xE8, dst));
        try!(write_array(&self.techniques, 0x14, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let atp = try!(u16::deserialize(src));
        let mst = try!(u16::deserialize(src));
        let evp = try!(u16::deserialize(src));
        let hp = try!(u16::deserialize(src));
        let dfp = try!(u16::deserialize(src));
        let ata = try!(u16::deserialize(src));
        let lck = try!(u16::deserialize(src));
        let unk1 = try!(u16::deserialize(src));
        let unk2_1 = try!(u32::deserialize(src));
        let unk2_2 = try!(u32::deserialize(src));
        let level = try!(u32::deserialize(src));
        let exp = try!(u32::deserialize(src));
        let meseta = try!(u32::deserialize(src));
        let guildcard = try!(read_ascii_len(16, src));
        let unk3 = try!(u64::deserialize(src));
        let name_color = try!(u32::deserialize(src));
        let model = try!(u8::deserialize(src));
        try!(src.read(&mut [0; 15]));
        let name_color_checksum = try!(u32::deserialize(src));
        let section = try!(u8::deserialize(src));
        let class = try!(u8::deserialize(src));
        let v2flags = try!(u8::deserialize(src));
        let version = try!(u8::deserialize(src));
        let v1flags = try!(u32::deserialize(src));
        let costume = try!(u16::deserialize(src));
        let skin = try!(u16::deserialize(src));
        let face = try!(u16::deserialize(src));
        let head = try!(u16::deserialize(src));
        let hair = try!(u16::deserialize(src));
        let hair_r = try!(u16::deserialize(src));
        let hair_g = try!(u16::deserialize(src));
        let hair_b = try!(u16::deserialize(src));
        let prop_x = try!(f32::deserialize(src));
        let prop_y = try!(f32::deserialize(src));
        let name = try!(read_utf16_len(32, src));
        let config = try!(read_array(0xE8, src));
        let techniques = try!(read_array(0x14, src));
        Ok(BbChar {
            atp: atp,
            mst: mst,
            evp: evp,
            hp: hp,
            dfp: dfp,
            ata: ata,
            lck: lck,
            unk1: unk1,
            unk2_1: unk2_1,
            unk2_2: unk2_2,
            level: level,
            exp: exp,
            meseta: meseta,
            guildcard: guildcard,
            unk3: unk3,
            name_color: name_color,
            model: model,
            name_color_checksum: name_color_checksum,
            section: section,
            class: class,
            v2flags: v2flags,
            version: version,
            v1flags: v1flags,
            costume: costume,
            skin: skin,
            face: face,
            head: head,
            hair: hair,
            hair_r: hair_r,
            hair_g: hair_g,
            hair_b: hair_b,
            prop_x: prop_x,
            prop_y: prop_y,
            name: name,
            config: config,
            techniques: techniques
        })
    }
}
impl Default for BbChar {
    fn default() -> Self {
        BbChar {
            atp: 0,
            mst: 0,
            evp: 0,
            hp: 0,
            dfp: 0,
            ata: 0,
            lck: 0,
            unk1: 0,
            unk2_1: 0,
            unk2_2: 0,
            level: 0,
            exp: 0,
            meseta: 0,
            guildcard: "".to_string(),
            unk3: 0,
            name_color: 0,
            model: 0,
            name_color_checksum: 0,
            section: 0,
            class: 0,
            v2flags: 0,
            version: 0,
            v1flags: 0,
            costume: 0,
            skin: 0,
            face: 0,
            head: 0,
            hair: 0,
            hair_r: 0,
            hair_g: 0,
            hair_b: 0,
            prop_x: 0.0,
            prop_y: 0.0,
            name: "".to_string(),
            config: vec![0; 0xE8],
            techniques: vec![0; 0x14]
        }
    }
}

#[derive(Clone, Debug)]
pub struct BbFullCharData {
    pub inv: Inventory,
    pub chara: BbChar,
    pub unk: Vec<u8>,
    pub option_flags: u32,
    pub quest_data1: Vec<u8>,
    pub bank: ItemBank,
    pub guildcard: u32,
    pub name: String,
    pub team_name: String,
    pub guildcard_desc: String,
    pub reserved1: u8,
    pub reserved2: u8,
    pub section: u8,
    pub class: u8,
    pub unk2: u32,
    pub symbol_chats: Vec<u8>,
    pub shortcuts: Vec<u8>,
    pub autoreply: String,
    pub infoboard: String,
    pub unk3: Vec<u8>,
    pub challenge_data: Vec<u8>,
    pub tech_menu: Vec<u8>,
    pub unk4: Vec<u8>,
    pub quest_data2: Vec<u8>,
    pub key_config: BbTeamAndKeyData
}
impl Serial for BbFullCharData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.inv.serialize(dst));
        try!(self.chara.serialize(dst));
        try!(write_array(&self.unk, 16, dst));
        try!(self.option_flags.serialize(dst));
        try!(write_array(&self.quest_data1, 520, dst));
        try!(self.bank.serialize(dst));
        try!(self.guildcard.serialize(dst));
        try!(write_utf16_len(&self.name, 24*2, dst));
        try!(write_utf16_len(&self.guildcard_desc, 88*2, dst));
        try!(self.reserved1.serialize(dst));
        try!(self.reserved2.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.class.serialize(dst));
        try!(self.unk2.serialize(dst));
        try!(write_array(&self.symbol_chats, 1248, dst));
        try!(write_array(&self.shortcuts, 2624, dst));
        try!(write_utf16_len(&self.autoreply, 172*2, dst));
        try!(write_utf16_len(&self.infoboard, 172*2, dst));
        try!(write_array(&self.unk3, 28, dst));
        try!(write_array(&self.challenge_data, 320, dst));
        try!(write_array(&self.tech_menu, 40, dst));
        try!(write_array(&self.unk4, 44, dst));
        try!(write_array(&self.quest_data2, 88, dst));
        try!(self.key_config.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let inv = try!(Serial::deserialize(src));
        let chara = try!(Serial::deserialize(src));
        let unk = try!(read_array(16, src));
        let option_flags = try!(Serial::deserialize(src));
        let quest_data1 = try!(read_array(520, src));
        let bank = try!(Serial::deserialize(src));
        let guildcard = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(24*2, src));
        let team_name = try!(read_utf16_len(16*2, src));
        let guildcard_desc = try!(read_utf16_len(88*2, src));
        let reserved1 = try!(Serial::deserialize(src));
        let reserved2 = try!(Serial::deserialize(src));
        let section = try!(Serial::deserialize(src));
        let class = try!(Serial::deserialize(src));
        let unk2 = try!(Serial::deserialize(src));
        let symbol_chats = try!(read_array(1248, src));
        let shortcuts = try!(read_array(2624, src));
        let autoreply = try!(read_utf16_len(172*2, src));
        let infoboard = try!(read_utf16_len(172*2, src));
        let unk3 = try!(read_array(28, src));
        let challenge_data = try!(read_array(320, src));
        let tech_menu = try!(read_array(40, src));
        let unk4 = try!(read_array(44, src));
        let quest_data2 = try!(read_array(88, src));
        let key_config = try!(Serial::deserialize(src));
        Ok(BbFullCharData {
            inv: inv,
            chara: chara,
            unk: unk,
            option_flags: option_flags,
            quest_data1: quest_data1,
            bank: bank,
            guildcard: guildcard,
            name: name,
            team_name: team_name,
            guildcard_desc: guildcard_desc,
            reserved1: reserved1,
            reserved2: reserved2,
            section: section,
            class: class,
            unk2: unk2,
            symbol_chats: symbol_chats,
            shortcuts: shortcuts,
            autoreply: autoreply,
            infoboard: infoboard,
            unk3: unk3,
            challenge_data: challenge_data,
            tech_menu: tech_menu,
            unk4: unk4,
            quest_data2: quest_data2,
            key_config: key_config
        })
    }
}
impl Default for BbFullCharData {
    fn default() -> Self {
        BbFullCharData {
            inv: Default::default(),
            chara: Default::default(),
            unk: vec![Default::default(); 16],
            option_flags: Default::default(),
            quest_data1: vec![Default::default(); 520],
            bank: Default::default(),
            guildcard: Default::default(),
            name: Default::default(),
            team_name: Default::default(),
            guildcard_desc: Default::default(),
            reserved1: Default::default(),
            reserved2: Default::default(),
            section: Default::default(),
            class: Default::default(),
            unk2: Default::default(),
            symbol_chats: vec![Default::default(); 1248],
            shortcuts: vec![Default::default(); 2624],
            autoreply: Default::default(),
            infoboard: Default::default(),
            unk3: vec![Default::default(); 28],
            challenge_data: vec![Default::default(); 320],
            tech_menu: vec![Default::default(); 40],
            unk4: vec![Default::default(); 44],
            quest_data2: vec![Default::default(); 88],
            key_config: Default::default()
        }
    }
}

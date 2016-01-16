use psoserial::Serial;
use psoserial::util::*;

use std::io;
use std::io::{Read, Write};

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Debug)]
pub struct InvItem {
    pub exists: u16,
    pub tech: u16,
    pub flags: u32,
    pub data: ItemData
}
impl Serial for InvItem {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.exists.serialize(dst));
        try!(self.tech.serialize(dst));
        try!(self.flags.serialize(dst));
        try!(self.data.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let exists = try!(Serial::deserialize(src));
        let tech = try!(Serial::deserialize(src));
        let flags = try!(Serial::deserialize(src));
        let data = try!(ItemData::deserialize(src));
        Ok(InvItem {
            exists: exists,
            tech: tech,
            flags: flags,
            data: data
        })
    }
}

impl Default for InvItem {
    fn default() -> InvItem {
        InvItem {
            exists: 0x00,
            tech: 0,
            flags: 0,
            data: Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BankItem {
    pub data: ItemData,
    pub amount: u16,
    pub flags: u16
}
impl Serial for BankItem {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.data.serialize(dst));
        try!(self.amount.serialize(dst));
        try!(self.flags.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let data = try!(Serial::deserialize(src));
        let amount = try!(Serial::deserialize(src));
        let flags = try!(Serial::deserialize(src));
        Ok(BankItem {
            data: data,
            amount: amount,
            flags: flags
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
            item_id: 0xFFFFFFFF,
            data2: vec![0; 4]
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub hp_mats: u8,
    pub tp_mats: u8,
    pub lang: u8,
    pub items: Vec<InvItem>
}
impl Serial for Inventory {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!((self.items.len() as u8).serialize(dst));
        try!(self.hp_mats.serialize(dst));
        try!(self.tp_mats.serialize(dst));
        try!(self.lang.serialize(dst));
        try!(write_array(&self.items, 30, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let _ = try!(u8::deserialize(src));
        let hp_mats = try!(u8::deserialize(src));
        let tp_mats = try!(u8::deserialize(src));
        let lang = try!(u8::deserialize(src));
        let mut items: Vec<InvItem> = try!(read_array(30, src));
        // Our representation will empty out unused slots, effectively
        // reorganizing them.
        items.retain(|i| i.exists != 0);
        Ok(Inventory {
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
            hp_mats: 0,
            tp_mats: 0,
            lang: 0,
            items: Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemBank {
    pub item_count: u32,
    pub meseta: u32,
    pub items: Vec<BankItem>
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
            items: vec![Default::default(); 200]
        }
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub struct CharStats {
    pub atp: u16,
    pub mst: u16,
    pub evp: u16,
    pub hp: u16,
    pub dfp: u16,
    pub lck: u16,
    pub ata: u16
}
impl Serial for CharStats {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.atp.serialize(dst));
        try!(self.mst.serialize(dst));
        try!(self.evp.serialize(dst));
        try!(self.hp.serialize(dst));
        try!(self.dfp.serialize(dst));
        try!(self.ata.serialize(dst));
        try!(self.lck.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let atp: u16 = try!(Serial::deserialize(src));
        let mst: u16 = try!(Serial::deserialize(src));
        let evp: u16 = try!(Serial::deserialize(src));
        let hp: u16 = try!(Serial::deserialize(src));
        let dfp: u16 = try!(Serial::deserialize(src));
        let ata: u16 = try!(Serial::deserialize(src));
        let lck: u16 = try!(Serial::deserialize(src));
        Ok(CharStats {
            atp: atp,
            mst: mst,
            evp: evp,
            hp: hp,
            dfp: dfp,
            lck: lck,
            ata: ata
        })
    }
}

#[derive(Clone, Debug)]
pub struct BbChar {
    pub stats: CharStats,
    pub unk1: u16,
    pub unk2_1: u32,
    pub unk2_2: u32,
    pub level: u32,
    pub exp: u32,
    pub meseta: u32,
    pub guildcard: String,
    pub unk3: Vec<u8>,
    pub name_color: u32,
    pub model: u8,
    pub unk5: Vec<u8>,
    pub name_color_checksum: u32,
    pub section: u8,
    pub class: u8,
    pub model_flag: u8,
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
    pub play_time: u32,
    pub unk4: Vec<u8>,
    pub config: Vec<u8>,
    pub techniques: Vec<u8>
}
impl Serial for BbChar {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(self.stats.serialize(dst));
        try!(self.unk1.serialize(dst));
        try!(self.unk2_1.serialize(dst));
        try!(self.unk2_2.serialize(dst));
        try!(self.level.serialize(dst));
        try!(self.exp.serialize(dst));
        try!(self.meseta.serialize(dst));
        try!(write_ascii_len(&self.guildcard, 10, dst));
        try!(write_array(&self.unk3, 14, dst));
        try!(self.name_color.serialize(dst));
        try!(self.model.serialize(dst));
        try!(write_array(&self.unk5, 15, dst));
        try!(self.name_color_checksum.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.class.serialize(dst));
        try!(self.model_flag.serialize(dst));
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
        try!(write_utf16_len(&self.name, 24, dst));
        try!(self.play_time.serialize(dst));
        try!(write_array(&self.unk4, 4, dst));
        try!(write_array(&self.config, 0xE8, dst));
        try!(write_array(&self.techniques, 0x14, dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let stats = try!(Serial::deserialize(src));
        let unk1 = try!(u16::deserialize(src));
        let unk2_1 = try!(u32::deserialize(src));
        let unk2_2 = try!(u32::deserialize(src));
        let level = try!(u32::deserialize(src));
        let exp = try!(u32::deserialize(src));
        let meseta = try!(u32::deserialize(src));
        let guildcard = try!(read_ascii_len(10, src));
        let unk3 = try!(read_array(14, src));
        let name_color = try!(u32::deserialize(src));
        let model = try!(u8::deserialize(src));
        let unk5 = try!(read_array(15, src));
        let name_color_checksum = try!(u32::deserialize(src));
        let section = try!(u8::deserialize(src));
        let class = try!(u8::deserialize(src));
        let model_flag = try!(u8::deserialize(src));
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
        let name = try!(read_utf16_len(24, src));
        let play_time = try!(Serial::deserialize(src));
        let unk4 = try!(read_array(4, src));
        let config = try!(read_array(0xE8, src));
        let techniques = try!(read_array(0x14, src));
        Ok(BbChar {
            stats: stats,
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
            unk5: unk5,
            name_color_checksum: name_color_checksum,
            section: section,
            class: class,
            model_flag: model_flag,
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
            play_time: play_time,
            unk4: unk4,
            config: config,
            techniques: techniques
        })
    }
}
impl Default for BbChar {
    fn default() -> Self {
        BbChar {
            stats: Default::default(),
            unk1: 0x28,
            unk2_1: 0x418C0000,
            unk2_2: 0x41200000,
            level: 0,
            exp: 0,
            meseta: 0x12C,
            guildcard: "  40000000".to_string(),
            unk3: vec![0u8; 14],
            name_color: 0,
            model: 0,
            unk5: vec![0; 15],
            play_time: 0,
            name_color_checksum: 0,
            section: 0,
            class: 0,
            model_flag: 0,
            version: 3,
            v1flags: 0x25,
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
            name: "\tEASH".to_string(),
            unk4: vec![0; 4],
            config: vec![
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00,
        	0x02, 0x01, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
        	0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
        	0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
        	0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
        	0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        	0x00, 0x00, 0x00, 0x00],
            techniques: vec![0xFF; 0x14]
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
        try!(write_array(&self.unk, 0x0010, dst));
        try!(self.option_flags.serialize(dst));
        try!(write_array(&self.quest_data1, 0x0208, dst));
        try!(self.bank.serialize(dst));
        try!(self.guildcard.serialize(dst));
        try!(write_utf16_len(&self.name, 0x18*2, dst));
        try!(write_utf16_len(&self.team_name, 0x10*2, dst));
        try!(write_utf16_len(&self.guildcard_desc, 0x58*2, dst));
        try!(self.reserved1.serialize(dst));
        try!(self.reserved2.serialize(dst));
        try!(self.section.serialize(dst));
        try!(self.class.serialize(dst));
        try!(self.unk2.serialize(dst));
        try!(write_array(&self.symbol_chats, 0x04E0, dst));
        try!(write_array(&self.shortcuts, 0x0A40, dst));
        try!(write_utf16_len(&self.autoreply, 0x00AC*2, dst));
        try!(write_utf16_len(&self.infoboard, 0x00AC*2, dst));
        try!(write_array(&self.unk3, 0x001C, dst));
        try!(write_array(&self.challenge_data, 0x0140, dst));
        try!(write_array(&self.tech_menu, 0x0028, dst));
        try!(write_array(&self.unk4, 0x002C, dst));
        try!(write_array(&self.quest_data2, 0x0058, dst));
        try!(self.key_config.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let inv = try!(Serial::deserialize(src));
        let chara = try!(Serial::deserialize(src));
        let unk = try!(read_array(0x0010, src));
        let option_flags = try!(Serial::deserialize(src));
        let quest_data1 = try!(read_array(0x0208, src));
        let bank = try!(Serial::deserialize(src));
        let guildcard = try!(Serial::deserialize(src));
        let name = try!(read_utf16_len(0x18*2, src));
        let team_name = try!(read_utf16_len(0x10*2, src));
        let guildcard_desc = try!(read_utf16_len(0x58*2, src));
        let reserved1 = try!(Serial::deserialize(src));
        let reserved2 = try!(Serial::deserialize(src));
        let section = try!(Serial::deserialize(src));
        let class = try!(Serial::deserialize(src));
        let unk2 = try!(Serial::deserialize(src));
        let symbol_chats = try!(read_array(0x04E0, src));
        let shortcuts = try!(read_array(0x0A40, src));
        let autoreply = try!(read_utf16_len(0x00AC*2, src));
        let infoboard = try!(read_utf16_len(0x00AC*2, src));
        let unk3 = try!(read_array(0x001C, src));
        let challenge_data = try!(read_array(0x0140, src));
        let tech_menu = try!(read_array(0x0028, src));
        let unk4 = try!(read_array(0x002C, src));
        let quest_data2 = try!(read_array(0x0058, src));
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
            unk: vec![0; 0x0010],
            option_flags: 0,
            quest_data1: vec![0; 0x0208],
            bank: Default::default(),
            guildcard: Default::default(),
            name: Default::default(),
            team_name: Default::default(),
            guildcard_desc: Default::default(),
            reserved1: 1,
            reserved2: 1,
            section: Default::default(),
            class: Default::default(),
            unk2: Default::default(),
            symbol_chats: super::bb_defaults::DEFAULT_SYMBOLCHATS.to_vec(),
            shortcuts: vec![Default::default(); 0x0A40],
            autoreply: Default::default(),
            infoboard: Default::default(),
            unk3: vec![0; 0x001C],
            challenge_data: vec![Default::default(); 0x0140],
            tech_menu: vec![
            0x00, 0x00, 0x06, 0x00, 0x03, 0x00, 0x01, 0x00, 0x07, 0x00, 0x04, 0x00,
        	0x02, 0x00, 0x08, 0x00, 0x05, 0x00, 0x09, 0x00, 0x12, 0x00, 0x0F, 0x00,
        	0x10, 0x00, 0x11, 0x00, 0x0D, 0x00, 0x0A, 0x00, 0x0B, 0x00, 0x0C, 0x00,
        	0x0E, 0x00, 0x00, 0x00],

            unk4: vec![0; 0x002C],
            quest_data2: vec![0; 0x0058],
            key_config: Default::default()
        }
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
    pub team_priv: u16,
    pub team_name: String,
    pub team_flag: Vec<u8>,
    pub team_rewards: u32
}
impl Serial for BbTeamAndKeyData {
    fn serialize(&self, dst: &mut Write) -> io::Result<()> {
        try!(write_array(&self.unk, 276, dst));
        try!(write_array(&self.key_config, 364, dst));
        try!(write_array(&self.joy_config, 56, dst));
        try!(dst.write_u32::<LE>(self.guildcard));
        try!(dst.write_u32::<LE>(self.team_id));
        try!(dst.write_u32::<LE>(self.team_info.0));
        try!(dst.write_u32::<LE>(self.team_info.1));
        try!(dst.write_u16::<LE>(self.team_priv));
        try!(dst.write_u16::<LE>(0));
        try!(write_utf16_len(&self.team_name, 28, dst));
        try!(0x00986C84u32.serialize(dst));
        try!(write_array(&self.team_flag, 2048, dst));
        try!(self.team_rewards.serialize(dst));
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<BbTeamAndKeyData> {
        let unk = try!(read_array(276, src));
        let key_config = try!(read_array(364, src));
        let joy_config = try!(read_array(56, src));
        let guildcard = try!(src.read_u32::<LE>());
        let team_id = try!(src.read_u32::<LE>());
        let team_info = (try!(src.read_u32::<LE>()), try!(src.read_u32::<LE>()));
        let team_priv = try!(src.read_u16::<LE>());
        try!(src.read_u16::<LE>());
        let team_name = try!(read_utf16_len(28, src));
        try!(u32::deserialize(src));
        let team_flag = try!(read_array(2048, src));
        let team_rewards = try!(Serial::deserialize(src));
        Ok(BbTeamAndKeyData {
            unk: unk,
            key_config: key_config,
            joy_config: joy_config,
            guildcard: guildcard,
            team_id: team_id,
            team_info: team_info,
            team_priv: team_priv,
            team_name: team_name,
            team_flag: team_flag,
            team_rewards: team_rewards
        })
    }
}

impl Default for BbTeamAndKeyData {
    fn default() -> BbTeamAndKeyData {
        BbTeamAndKeyData {
            // TODO actual BB defaults for these
            unk: vec![0u8; 276],
            key_config: super::bb_defaults::DEFAULT_KEYS.to_vec(),
            joy_config: super::bb_defaults::DEFAULT_JOY.to_vec(),
            guildcard: 0, // This is always 0 if the player isn't in a team.
            team_id: 0,
            team_info: (0, 0),
            team_priv: 0,
            team_name: "".to_string(), // Must be zeroed out when not in a team
            team_flag: vec![0; 2048],
            team_rewards: 0xFFFFFFFF
        }
    }
}

pub mod preview {
    use psoserial::util::*;
    use psoserial::Serial;

    use std::io;
    use std::io::{Write, Read};

    use byteorder::{LittleEndian as LE, ReadBytesExt};

    #[derive(Clone, Debug)]
    pub struct BbMiniCharData {
        pub exp: u32,
        pub level: u32,
        pub guildcard: String,
        pub name_color: u32,
        pub model: u8,
        pub name_color_checksum: u32,
        pub section: u8,
        pub class: u8,
        pub model_flag: u8,
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
        pub name: String, // 8 characters of some unknown encoding
        pub play_time: u32
    }
    impl Serial for BbMiniCharData {
        fn serialize(&self, dst: &mut Write) -> io::Result<()> {
            try!(self.exp.serialize(dst));
            try!(self.level.serialize(dst));
            try!(write_ascii_len(&self.guildcard, 16, dst));
            try!(dst.write_all(&[0; 8]));
            try!(self.name_color.serialize(dst));
            try!(self.model.serialize(dst));
            try!(dst.write_all(&[0; 15]));
            try!(self.name_color_checksum.serialize(dst));
            try!(self.section.serialize(dst));
            try!(self.class.serialize(dst));
            try!(self.model_flag.serialize(dst));
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
            try!(write_utf16_len(&self.name, 24, dst));
            try!(self.play_time.serialize(dst));
            try!(0u32.serialize(dst));
            try!(self.play_time.serialize(dst));
            Ok(())
        }

        fn deserialize(src: &mut Read) -> io::Result<Self> {
            let exp = try!(src.read_u32::<LE>());
            let level = try!(src.read_u32::<LE>());
            let guildcard = try!(read_ascii_len(16, src));
            try!(src.read(&mut [0; 8]));
            let name_color = try!(src.read_u32::<LE>());
            let model = try!(src.read_u8());
            try!(src.read(&mut [0; 15]));
            let name_color_checksum = try!(src.read_u32::<LE>());
            let section = try!(src.read_u8());
            let class = try!(src.read_u8());
            let model_flag = try!(src.read_u8());
            let version = try!(src.read_u8());
            let v1flags = try!(src.read_u32::<LE>());
            let costume = try!(src.read_u16::<LE>());
            let skin = try!(src.read_u16::<LE>());
            let face = try!(src.read_u16::<LE>());
            let head = try!(src.read_u16::<LE>());
            let hair = try!(src.read_u16::<LE>());
            let hair_r = try!(src.read_u16::<LE>());
            let hair_g = try!(src.read_u16::<LE>());
            let hair_b = try!(src.read_u16::<LE>());
            let prop_x = try!(src.read_f32::<LE>());
            let prop_y = try!(src.read_f32::<LE>());
            let name = try!(read_utf16_len(24, src));
            try!(u32::deserialize(src));
            try!(u32::deserialize(src));
            let play_time = try!(src.read_u32::<LE>());
            Ok(BbMiniCharData {
                exp: exp,
                level: level,
                guildcard: guildcard,
                name_color: name_color,
                model: model,
                name_color_checksum: name_color_checksum,
                section: section,
                class: class,
                model_flag: model_flag,
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
                play_time: play_time
            })
        }
    }

    impl Default for BbMiniCharData {
        fn default() -> BbMiniCharData {
            BbMiniCharData {
                exp: 0,
                level: 0,
                guildcard: "  ".to_string(),
                name_color: 0xFFFFFFFF,
                model: 0,
                name_color_checksum: 0,
                section: 0,
                class: 0,
                model_flag: 0,
                version: 3,
                v1flags: 0x25,
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
                name: "\tEDefault".to_string(),
                play_time: 0
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use psoserial::Serial;
    use super::*;

    #[test]
    fn test_full_char_size() {
        let mut cursor = Cursor::new(Vec::new());
        let ch = BbFullCharData::default();
        ch.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 0x3994);
    }

    #[test]
    fn test_inv_item_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = InvItem::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 28);
    }

    #[test]
    fn test_bank_item_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = BankItem::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 24);
    }

    #[test]
    fn test_inventory_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = Inventory::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 4 + 30*28);
    }

    #[test]
    fn test_bank_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = ItemBank::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 8 + 200*24);
    }

    #[test]
    fn test_bb_char_size() {
        let mut cursor = Cursor::new(Vec::new());
        let a = BbChar::default();
        a.serialize(&mut cursor).unwrap();
        assert_eq!(cursor.position(), 400);
    }
}

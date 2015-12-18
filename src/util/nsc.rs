use std::io;
use std::io::Read;

use psomsg::Serial;
use psomsg::util::*;
use psomsg::bb::chara::*;
use psomsg::bb::data::BbMiniCharData;

use ::game::CharClass;

/// Read a Fuzziqer newserv NSC character save file.
pub fn read_nsc(r: &mut Read, class: CharClass) -> io::Result<BbFullCharData> {
    let _: Vec<u8> = try!(read_array(0x40, r));
    let _: BbMiniCharData = try!(Serial::deserialize(r));
    let autoreply = try!(read_utf16_len(0xAC*2, r));
    let bank: ItemBank = try!(Serial::deserialize(r));
    let challenge: Vec<u8> = try!(read_array(0x140, r));
    let disp: BbChar = try!(Serial::deserialize(r));
    let guildcard_desc = try!(read_utf16_len(0x58*2, r));
    let infoboard = try!(read_utf16_len(0xAC*2, r));
    let inventory: Inventory = try!(Serial::deserialize(r));
    let quest_data1: Vec<u8> = try!(read_array(0x0208, r));
    let quest_data2: Vec<u8> = try!(read_array(0x58, r));
    let tech_config: Vec<u8> = try!(read_array(0x28, r));

    let mut r = BbFullCharData::default();
    r.inv = inventory;
    r.chara = disp;
    r.quest_data1 = quest_data1;
    r.bank = bank;
    r.name = r.chara.name.clone();
    r.guildcard_desc = guildcard_desc;
    r.reserved1 = 1;
    r.reserved2 = 1;
    r.class = class as u8;
    r.autoreply = autoreply;
    r.infoboard = infoboard;
    r.challenge_data = challenge;
    r.tech_menu = tech_config;
    r.quest_data2 = quest_data2;
    Ok(r)
}

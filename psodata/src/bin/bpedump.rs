extern crate psodata;

use std::io::{Read};
use std::fs::File;

use psodata::{Parse, BattleParam};

fn main() {
    let mut bpefile = File::open("C:/Users/Eidolon/Code/rust/idolapsoserv/data/param/BattleParamEntry.dat").unwrap();
    let mut entries = Vec::new();

    for _ in 0..0x180 {
        entries.push(BattleParam::read(&mut bpefile).unwrap());
    }

    println!("BattleParamEntry.dat");
    println!("INDX ATP    INT    EVP    HP     DFP    ATA    LCK    EXP    DIFF  ");
    for (x, i) in entries.iter().enumerate() {
        println!("{:4} {:6} {:6} {:6} {:6} {:6} {:6} {:6} {:6} {:6}",
            x, i.atp, i.int, i.evp, i.hp, i.dfp, i.ata, i.lck, i.exp, i.difficulty);
    }
}

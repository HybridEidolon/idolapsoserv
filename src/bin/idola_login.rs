extern crate idola;
extern crate psocrypto;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate encoding;

use std::net::*;
use std::fs::File;
//use std::thread;
use std::sync::Arc;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init().unwrap();

    // Read 1042 BB key table from data/crypto

    let key_table = Arc::new(idola::bb::read_key_table(&mut File::open("data/crypto/bb_table.bin").unwrap()).unwrap());
    info!("Read Blue Burst encryption key table from data/crypto/bb_table.bin");
    debug!("The first few values are {:x}, {:x}, {:x}, {:x}", key_table[0], key_table[1], key_table[2], key_table[3]);

    // make db
    //let db_pool = Arc::new(Pool::new(1, &mut Sqlite::new("test.db", true).unwrap()).unwrap());

    let tcp_listener = TcpListener::bind("0.0.0.0:12000").unwrap();
    for s in tcp_listener.incoming() {
        match s {
            Ok(_) => {
                //use idola::login::bb::{Context, run_login};
                //let kt_clone = key_table.clone();
                //let db_clone = db_pool.clone();
                unimplemented!()
                //thread::spawn(move|| run_login(Context::new(s, kt_clone, db_clone, None)));
            },
            Err(e) => error!("error, quitting: {}", e)
        }
    }
}

extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;

use idola::patch::PatchServer;

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init().unwrap();

    info!("IDOLA Phantasy Star Online Patch Server");
    info!("Version 0.1.0");

    // TODO read config

    let s = PatchServer::new_bb("AAAAAHHH!!!.... how am I gonna feed all these lil... BABS!\n\n\nyou are client {client_num} to connect", "0.0.0.0:11000");

    s.run().unwrap();

    info!("Patch server is closing. Bye!");
}

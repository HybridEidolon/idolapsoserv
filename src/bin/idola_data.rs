//! Runs the data provision server, as a component of the patch server.
extern crate rand;
extern crate psocrypto;
extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate byteorder;

use idola::patch::DataServer;

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init().unwrap();

    info!("IDOLA Phantasy Star Online Data Server");
    info!("Version 0.1.0");

    DataServer::new_bb("127.0.0.1:11001").run().unwrap();

    info!("Data server going down.");
}

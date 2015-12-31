// Separated compile units
extern crate psocrypto;
extern crate psomsg;
extern crate psodata;
extern crate psodb_common;
extern crate psodb_sqlite;

extern crate rand;
extern crate byteorder;
extern crate encoding;
extern crate typenum;
#[macro_use] extern crate log;
extern crate crc;
extern crate docopt;
extern crate mio;
extern crate rustc_serialize;
extern crate staticvec;
extern crate env_logger;
extern crate toml;

pub mod patch;
pub mod data;
pub mod game;
pub mod login;
pub mod bb;
pub mod ship;
pub mod util;
pub mod args;
pub mod loop_handler;
pub mod services;
pub mod config;

use ::args::USAGE_STRING;
use ::args::Args;

use docopt::Docopt;

use mio::EventLoop;

use ::loop_handler::LoopHandler;
use ::patch::PatchService;
use ::data::DataService;
use ::login::bb::BbLoginService;

use ::config::Config;

use std::fs::File;
use std::sync::Arc;

use ::game::Version;
use ::bb::read_key_table;

fn main() {
    env_logger::init().expect("env_logger failed to initialize");

    let args: Args = Docopt::new(USAGE_STRING)
        .and_then(|o| o.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("idola 0.1.0");
        return
    }

    let config;
    {
        use std::io::Read;
        let mut config_file = File::open(&args.flag_config).expect(&format!("Failed to open file {}", args.flag_config));
        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).expect("Failed to read config file completely");
        config = Config::from_toml_string(&config_string).expect("Failed to parse TOML");
    }

    // Load the bb key table.
    let bb_keytable;
    {
        let mut keytable_file = File::open(&config.bb_keytable_path).expect("Failed to open BB keytable file");
        bb_keytable = Arc::new(read_key_table(&mut keytable_file).expect("Failed to parse BB keytable"));
    }

    // 1. Create EventLoop.
    // 2. Spin up service threads to get handles.
    // 3. Create LoopHandler and register services.
    // 4. Run event loop.
    let mut event_loop = EventLoop::new().expect("Could not create event loop");

    let mut services = Vec::new();
    for s in config.services.iter() {
        use ::config::ServiceConf;
        match s {
            &ServiceConf::Patch { ref bind, ref v4_servers, ref motd, random_balance } => {
                println!("Patch service at {:?}", bind);
                services.push(PatchService::spawn(bind, event_loop.channel(), v4_servers.clone(), motd.clone(), random_balance));
            },
            &ServiceConf::Data { ref bind, .. } => {
                println!("Data service at {:?}", bind);
                services.push(DataService::spawn(bind, event_loop.channel()));
            },
            &ServiceConf::Login { ref bind, version, .. } => {
                println!("Login service at {:?}", bind);
                match version {
                    Version::BlueBurst => {
                        services.push(BbLoginService::spawn(bind, event_loop.channel(), bb_keytable.clone()))
                    },
                    _ => unimplemented!()
                }
            }
        }
    }
    println!("{} total services.", services.len());

    let mut loop_handler = LoopHandler::new(services, &mut event_loop);

    event_loop.run(&mut loop_handler).unwrap();
}

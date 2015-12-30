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

pub mod patch;
pub mod game;
pub mod login;
pub mod bb;
pub mod ship;
pub mod util;
pub mod args;
pub mod loop_handler;
pub mod services;

use ::args::USAGE_STRING;
use ::args::Args;

use docopt::Docopt;

use mio::EventLoop;

use ::loop_handler::LoopHandler;
use ::services::patch::PatchService;

fn main() {
    let args: Args = Docopt::new(USAGE_STRING)
        .and_then(|o| o.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("idola 0.1.0");
        return
    }

    // TODO load the config

    // 1. Create EventLoop.
    // 2. Spin up service threads to get handles.
    // 3. Create LoopHandler and register services.
    // 4. Run event loop.
    let mut event_loop = EventLoop::new().expect("Could not create event loop");

    let mut services = Vec::new();
    services.push(PatchService::spawn(&"127.0.0.1:11000".parse().unwrap(), event_loop.channel()));

    let mut loop_handler = LoopHandler::new(services, &mut event_loop);

    event_loop.run(&mut loop_handler).unwrap();
}

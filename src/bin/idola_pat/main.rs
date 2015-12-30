//! Patch server.

extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;
extern crate toml;
extern crate mio;

extern crate psomsg;
extern crate psocrypto;

mod loop_handler;
mod services;
mod doc;

use ::loop_handler::LoopHandler;
use ::doc::{USAGE_STRING, Args};

use ::services::patch::PatchService;

use mio::EventLoop;

use docopt::Docopt;

fn main() {
    let _: Args = Docopt::new(USAGE_STRING)
        .and_then(|o| o.decode())
        .unwrap_or_else(|e| e.exit());

    // 1. Create EventLoop.
    // 2. Spin up service threads to get handles.
    // 3. Create LoopHandler and register services.
    // 4. Run event loop.
    let mut event_loop = EventLoop::new().expect("Could not create event loop");

    let mut services = Vec::new();
    services.push(PatchService::spawn(&"127.0.0.1:11000".parse().unwrap(), event_loop.channel()));
    println!("{}", services.len());

    let mut loop_handler = LoopHandler::new(services, &mut event_loop);

    event_loop.run(&mut loop_handler).unwrap();
}

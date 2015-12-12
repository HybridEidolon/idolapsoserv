extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;
extern crate toml;

use idola::patch::PatchServer;

use std::fs::File;
use std::io::Read;
use std::net::{SocketAddrV4, Ipv4Addr};
use std::str::FromStr;

use docopt::Docopt;
use toml::Parser;
use rustc_serialize::Decodable;

const USAGE: &'static str = "
IDOLA Phantasy Star Online Patch Server

Usage:
  idola_patch [options]
  idola_patch (-h | --help)
  idola_patch --version

Options:
  -h --help          Show this message.
  --version          Show version.
  --config=<config>  Set the path to load TOML configuration from. Default is \"patch_config.toml\"
  --bind=<addr>      Set the bind address for this patch server. Overrides config TOML.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_config: Option<String>,
    flag_addr: Option<String>,
    flag_version: bool
}

#[derive(Debug, RustcDecodable)]
struct Config {
    bind: Option<String>,
    motd_template: String,
    data_servers: Vec<ConfigDataServer>
}

#[derive(Debug, RustcDecodable)]
struct ConfigDataServer {
    pub ip: String,
    pub port: u16
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind: None,
            motd_template: "Client {client_num} connected. This patch server has not set a MOTD!".to_string(),
            data_servers: Vec::new()
        }
    }
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("idola_patch 0.1.0");
        return
    }

    let config_path = args.flag_config.unwrap_or("patch_config.toml".to_string());
    let mut config_contents = String::new();
    File::open(config_path).unwrap().read_to_string(&mut config_contents).unwrap();
    let config;
    {
        let mut parser = toml::Parser::new(&config_contents);
        let fields = match parser.parse() {
            None => panic!("Parsing config failed: {:?}", parser.errors),
            Some(s) => match s.get("config") {
                None => panic!("Config did not have a [config] block"),
                Some(f) => f.clone()
            }
        };
        config = Config::decode(&mut toml::Decoder::new(fields)).unwrap();
    }
    println!("{:?}", config);

    let data_servers: Vec<_> = config.data_servers.iter().map(|d| {
        SocketAddrV4::new(Ipv4Addr::from_str(&d.ip).unwrap(), d.port)
    }).collect();

    let s = PatchServer::new_bb(config.motd_template, config.bind.unwrap_or("0.0.0.0:11000".to_string()), &data_servers);

    s.run().unwrap();

    info!("Patch server is closing. Bye!");
}

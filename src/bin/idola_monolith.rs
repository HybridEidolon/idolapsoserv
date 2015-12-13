extern crate idola;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;
extern crate toml;

use std::net::{Ipv4Addr, SocketAddrV4};

use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;

use docopt::Docopt;

const USAGE: &'static str = "
IDOLA Phantasy Star Online 'Monolith' Server
Provides a patch, data, login, and character server all in one.

Usage:
  idola_monolith [options]
  idola_monolith (-h | --help)
  idola_monolith --version

Options:
  -h, --help          Show this message.
  --version           Show the version.
  --config=<cfg>      Set the Monolith config path. Defaults to 'monolith_config.toml'
";


#[derive(Clone, Debug, RustcDecodable)]
struct Args {
    flag_config: Option<String>,
    flag_version: bool
}

#[derive(Debug, RustcDecodable)]
struct Config {
    motd_template: String,
    bind_address: Option<String>,
    bb_keytable_path: Option<String>
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MonolithMsg {
    Up(MonolithComponent),
    DownErr(MonolithComponent, String),
    DownGraceful(MonolithComponent)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MonolithComponent {
    Patch,
    Data,
    Login
}

fn patch_server(channel: Sender<MonolithMsg>, motd_template: String, bind_address: String) {
    use idola::patch::*;
    use std::str::FromStr;
    use std::error::Error;
    let bind_ipv4addr = Ipv4Addr::from_str("127.0.0.1").unwrap();
    let data_servers = vec![SocketAddrV4::new(bind_ipv4addr, 11001)];
    let server = PatchServer::new_bb(motd_template.clone(), bind_address.clone(), &data_servers);
    channel.send(MonolithMsg::Up(MonolithComponent::Patch)).unwrap();
    channel.send(match server.run() {
        Err(e) => MonolithMsg::DownErr(MonolithComponent::Patch, e.description().to_string()),
        Ok(_) => MonolithMsg::DownGraceful(MonolithComponent::Patch)
    }).unwrap();
}

fn data_server(channel: Sender<MonolithMsg>) {
    use idola::patch::*;
    let data_server = DataServer::new_bb("127.0.0.1:11001");
    channel.send(MonolithMsg::Up(MonolithComponent::Data)).unwrap();
    channel.send(match data_server.run() {
        Err(e) => MonolithMsg::DownErr(MonolithComponent::Data, e.to_string()),
        Ok(_) => MonolithMsg::DownGraceful(MonolithComponent::Data)
    }).unwrap();
}

fn login_server(channel: Sender<MonolithMsg>, key_table_path: String) {
    use std::sync::Arc;
    use std::fs::File;
    use std::net::TcpListener;
    use std::thread;
    use idola::db::Pool;
    use idola::db::sqlite::Sqlite;

    channel.send(MonolithMsg::Up(MonolithComponent::Login)).unwrap();
    let key_table: Arc<Vec<u32>> = Arc::new(idola::bb::read_key_table(&mut File::open(&key_table_path).unwrap()).unwrap());

    // make db
    let db_pool = Arc::new(Pool::new(1, &mut Sqlite::new("test.db", true).unwrap()).unwrap());

    let tcp_listener = TcpListener::bind("127.0.0.1:12000").unwrap();
    for s in tcp_listener.incoming() {
        match s {
            Ok(s) => {
                let kt_clone = key_table.clone();
                let db_clone = db_pool.clone();
                thread::spawn(move|| idola::bb::Context::new(s, kt_clone, db_clone).run().unwrap());
            },
            Err(e) => error!("error, quitting: {}", e)
        }
    }

    channel.send(MonolithMsg::DownGraceful(MonolithComponent::Login)).unwrap();
}

fn read_config(path: &str) -> Config {
    use std::fs::File;
    use std::io::Read;
    use rustc_serialize::Decodable;
    let mut config_contents = String::new();
    File::open(path).unwrap().read_to_string(&mut config_contents).unwrap();
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
    config
}

fn main() {
    use std::thread;

    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "DEBUG");
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("idola_monolith 0.1.0");
        return
    }

    let config = read_config(&args.flag_config.clone().unwrap_or("monolith_config.toml".to_string()));
    let motd_template_clone = config.motd_template.clone();
    let bb_keytable_clone = config.bb_keytable_path.clone();

    let (tx, rx) = channel();
    let tx_c = tx.clone();
    thread::spawn(move|| patch_server(tx_c, motd_template_clone, "127.0.0.1:11000".to_string()));
    let tx_c = tx.clone();
    thread::spawn(move|| data_server(tx_c));
    let tx_c = tx.clone();
    thread::spawn(move|| login_server(tx_c, bb_keytable_clone.clone().unwrap_or("data/crypto/bb_table.bin".to_string())));

    let mut patch_status = true;
    let mut data_status = true;
    let mut login_status = true;

    for m in rx.iter() {
        use MonolithComponent::*;
        match m {
            MonolithMsg::DownErr(a, s) => {
                match a {
                    Patch => patch_status = false,
                    Data => data_status = false,
                    Login => login_status = false
                }
                error!("Down by error {:?}: {:?}", a, s);
            },
            MonolithMsg::DownGraceful(a) => {
                match a {
                    Patch => patch_status = false,
                    Data => data_status = false,
                    Login => login_status = false
                }
                info!("{:?} down gracefully", a)
            },
            _ => ()
        }

        if !patch_status && !login_status && !data_status {
            break
        }
    }
}

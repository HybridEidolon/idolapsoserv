use std::fs::File;
use std::io::Read;

use toml::Parser;
use toml::decode;

#[derive(RustcDecodable, Clone, Debug)]
pub struct Config {
    pub server_keytable: String,
    pub client_keytable: String,
    pub server_patch_ip: String,
    pub server_login_ip: String,
    pub server_patch_port: u16,
    pub server_login_port: u16,
    pub client_patch_port: u16,
    pub client_login_port: u16,
    pub version_override: String,
    pub checksum_override: u32
}

pub fn read_config(path: &str) -> Option<Config> {
    let mut f = File::open(path).unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();
    let mut parser = Parser::new(&text);
    let toml = parser.parse().unwrap();
    decode(toml.get("proxy").unwrap().clone())
}

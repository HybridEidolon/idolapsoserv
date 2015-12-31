use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};

use toml::{Parser, Table};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_path: Option<String>,
    pub services: Vec<ServiceConf>
}

#[derive(Debug, Clone)]
pub enum ServiceConf {
    Patch {
        bind: SocketAddr,
        motd: String,
        v4_servers: Vec<SocketAddrV4>,
        random_balance: bool
    },
    Data {
        bind: SocketAddr
    },
    // ...
}

impl Config {
    pub fn from_toml_string(s: &str) -> Result<Config, String> {
        let mut parser = Parser::new(s);
        if let Some(value) = parser.parse() {
            Config::from_toml_value(&value)
        } else {
            Err(format!("{:?}", parser.errors))
        }
    }

    pub fn from_toml_value(t: &Table) -> Result<Config, String> {
        let data_path;
        if let Some(i) = t.get("idola") {
            data_path = i.lookup("data_path").and_then(|v| v.as_str()).map(|s| s.to_string());
        } else {
            return Err("No idola section".to_string())
        }
        let mut services = Vec::new();
        if let Some(s_slice) = t.get("service").and_then(|v| v.as_slice()) {
            for s in s_slice {
                match s.as_table() {
                    Some(stab) => services.push(try!(ServiceConf::from_toml_table(stab))),
                    None => return Err("a configured service is not a TOML table".to_string())
                }
            }
        }
        Ok(Config {
            data_path: data_path,
            services: services
        })
    }
}

impl ServiceConf {
    pub fn from_toml_table(t: &Table) -> Result<ServiceConf, String> {
        if let Some(bind) = t.get("bind").and_then(|v| v.as_str()).and_then(|s| s.to_socket_addrs().ok()).and_then(|mut s| s.next()) {
            if let Some(ty) = t.get("type").and_then(|v| v.as_str()) {
                match ty {
                    "patch" => {
                        let motd = t.get("motd").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
                        let random_balance = t.get("random_balance").and_then(|v| v.as_bool()).unwrap_or_default();
                        let mut v4_servers = Vec::new();
                        if let Some(v4_values) = t.get("v4_servers").and_then(|v| v.as_slice()) {
                            for v in v4_values {
                                if let Some(sockaddr) = v.as_str().and_then(|s| s.parse().ok()) {
                                    v4_servers.push(sockaddr);
                                } else {
                                    return Err("patch service's data address is not a valid IPv4 address:port string".to_string())
                                }
                            }
                        } else {
                            return Err("patch service v4_servers field is not an array".to_string())
                        }
                        if v4_servers.len() == 0 {
                            return Err("patch service has no IPv4 data nodes declared".to_string())
                        }
                        Ok(ServiceConf::Patch {
                            bind: bind,
                            motd: motd,
                            v4_servers: v4_servers,
                            random_balance: random_balance
                        })
                    },
                    "data" => {
                        Ok(ServiceConf::Data {
                            bind: bind
                        })
                    },
                    _ => return Err("invalid service type specified".to_string())
                }
            } else {
                Err("Service has no type declared".to_string())
            }
        } else {
            Err("No bind address specified for service".to_string())
        }
    }
}

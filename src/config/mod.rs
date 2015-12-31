use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};

use toml::{Parser, Table};

use ::game::Version;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_path: String,
    pub bb_keytable_path: String,
    pub shipgate_addr: SocketAddr,
    pub shipgate_password: String,
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
    Login {
        bind: SocketAddr,
        version: Version
        // The login service just redirects to one of the ship servers.
        // In this implementation, the ship servers act as a character server
        // for BB.
    },
    ShipGate {
        bind: SocketAddr,
        password: String,
        db: DbConf
    }
    // ...
}

#[derive(Debug, Clone)]
pub enum DbConf {
    Sqlite {
        file: String
    }
}

impl Config {
    pub fn from_toml_string(s: &str) -> Result<Config, String> {
        let mut parser = Parser::new(s);
        if let Some(value) = parser.parse() {
            Config::from_toml_value(&value)
        } else {
            let errors: Vec<String> = parser.errors.into_iter().map(|e| format!("{}", e)).collect();
            Err(format!("{:?}", errors))
        }
    }

    pub fn from_toml_value(t: &Table) -> Result<Config, String> {
        let data_path;
        let bb_keytable_path;
        let shipgate_addr;
        let shipgate_password;
        if let Some(i) = t.get("idola") {
            data_path = i.lookup("data_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or("data".to_string());
            bb_keytable_path = i.lookup("bb_keytable_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or(format!("{}/crypto/bb_table.bin", data_path));
            shipgate_addr = match i.lookup("shipgate_addr")
                .and_then(|v| v.as_str())
                .and_then(|s| s.to_socket_addrs().ok())
                .and_then(|mut s| s.next()) {
                    Some(v) => v,
                    None => return Err("Shipgate address not specified or malformed".to_string())
                };
            shipgate_password = match i.lookup("shipgate_password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()) {
                    Some(v) => v,
                    None => return Err("Shipgate password is not specified.".to_string())
                };
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
            bb_keytable_path: bb_keytable_path,
            services: services,
            shipgate_addr: shipgate_addr,
            shipgate_password: shipgate_password
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
                    "login" => {
                        let version;
                        match t.get("version")
                            .and_then(|v| v.as_str())
                            .map(|v| v.parse()) {
                            Some(Ok(v)) => version = v,
                            Some(Err(e)) => return Err(e),
                            None => return Err("No version specified for login service".to_string())
                        }
                        Ok(ServiceConf::Login {
                            bind: bind,
                            version: version
                        })
                    },
                    "shipgate" => {
                        let password;
                        let db;
                        if let Some(p) = t.get("password")
                            .and_then(|v| v.as_str())
                            .map(|v| v.to_string()) {
                            password = p;
                        } else {
                            return Err("No password for shipgate specified".to_string())
                        }
                        if let Some(d) = t.get("db")
                            .and_then(|v| v.as_table()) {
                            match DbConf::from_toml_table(d) {
                                Ok(dbv) => db = dbv,
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err("No db configured for shipgate".to_string())
                        }
                        Ok(ServiceConf::ShipGate {
                            bind: bind,
                            password: password,
                            db: db
                        })
                    }
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

impl DbConf {
    pub fn from_toml_table(t: &Table) -> Result<DbConf, String> {
        match t.get("type").and_then(|v| v.as_str()) {
            Some("sqlite") => {
                let file;
                if let Some(f) = t.get("file")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()) {
                    file = f;
                } else {
                    return Err("sqlite DB type file path missing.".to_string())
                }
                Ok(DbConf::Sqlite {
                    file: file
                })
            },
            Some(t) => { Err(format!("unsupported db type {}", t)) },
            None => { Err("shipgate db type not specified".to_string()) }
        }
    }
}

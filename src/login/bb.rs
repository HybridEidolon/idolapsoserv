//! Blue Burst Login and Character server structures. They're a pair much like the Patch and Data
//! servers are. Since each connecting client is not communicating with each other or with a
//! central service, the state machine functions will consume and drop the connection context
//! on its own.

use ::db::pool::Pool;

use std::net::{TcpStream, Ipv4Addr};

use std::sync::Arc;

use psocrypto::bb::BbCipher;
use psocrypto::{DecryptReader, EncryptWriter};
use rand::random;

use psomsg::bb::*;

pub struct Context {
    stream: TcpStream,
    key_table: Arc<Vec<u32>>,
    db_pool: Arc<Pool>,
    param_chunks: Option<Arc<(Message, Vec<Message>)>>
}

impl Context {
    pub fn new(stream: TcpStream, key_table: Arc<Vec<u32>>, db_pool: Arc<Pool>, param_chunks: Option<Arc<(Message, Vec<Message>)>>) -> Context {
        Context {
            stream: stream,
            key_table: key_table,
            db_pool: db_pool,
            param_chunks: param_chunks
        }
    }
}

/// Runs the Login server state machine on a context.
pub fn run_login(mut ctx: Context, char_ip: Ipv4Addr, char_port: u16) -> () {
    use psomsg::bb::*;
    let peer_addr = ctx.stream.peer_addr().unwrap();

    info!("[{}] login blue burst: connected", peer_addr);

    // make new ciphers
    let client_key = vec![random::<u8>(); 48];
    let server_key = vec![random::<u8>(); 48];
    let client_cipher = BbCipher::new(&client_key, &ctx.key_table);
    let server_cipher = BbCipher::new(&server_key, &ctx.key_table);

    let welcome = Message::Welcome(0, Welcome(server_key, client_key));

    welcome.serialize(&mut ctx.stream).unwrap();

    // now, wrap the stream with encrypt/decrypt
    let mut w_s = EncryptWriter::new(ctx.stream.try_clone().unwrap(), server_cipher);
    let mut r_s = DecryptReader::new(ctx.stream.try_clone().unwrap(), client_cipher);

    loop {
        let m = Message::deserialize(&mut r_s).unwrap();
        match m {
            Message::Unknown(o, f, b) => {
                info!("[{}] unknown message type 0x{:x}, flags {}, {:?}", peer_addr, o, f, b);
            },
            Message::Login(_, Login { username, password, .. }) => {
                match check_login(ctx.db_pool.clone(), &username, &password) {
                    CheckLogin::Success => {
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 0, // login success
                            tag: 0,
                            guildcard: 0,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                        // TODO redirect to character server
                        let r = Message::Redirect(0, Redirect {
                            ip: char_ip,
                            port: char_port
                        });
                        r.serialize(&mut w_s).unwrap();
                        return
                    },
                    CheckLogin::WrongPassword => {
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 2, // bad pw
                            tag: 0,
                            guildcard: 0,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                        return
                    },
                    CheckLogin::Banned => {
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 6, // banned
                            tag: 0,
                            guildcard: 0,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                        return
                    },
                    CheckLogin::NoAccount => {
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 8, // no account exists
                            tag: 0,
                            guildcard: 0,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                        return
                    }
                }
            }
            a => warn!("Received an unexpected but known message: {:?}", a)
        }
    }
}

/// Runs the Character server state machine on a context.
pub fn run_character(mut ctx: Context) -> () {
    use psomsg::bb::*;
    let peer_addr = ctx.stream.peer_addr().unwrap();

    let param_chunks = ctx.param_chunks.clone().unwrap();

    info!("[{}] character blue burst: connected", peer_addr);

    // make new ciphers
    let client_key = vec![random::<u8>(); 48];
    let server_key = vec![random::<u8>(); 48];
    let client_cipher = BbCipher::new(&client_key, &ctx.key_table);
    let server_cipher = BbCipher::new(&server_key, &ctx.key_table);

    let welcome = Message::Welcome(0, Welcome(server_key, client_key));

    welcome.serialize(&mut ctx.stream).unwrap();

    // now, wrap the stream with encrypt/decrypt
    let mut w_s = EncryptWriter::new(ctx.stream.try_clone().unwrap(), server_cipher);
    let mut r_s = DecryptReader::new(ctx.stream.try_clone().unwrap(), client_cipher);

    loop {
        let m = Message::deserialize(&mut r_s).unwrap();
        match m {
            Message::Unknown(o, f, b) => {
                info!("[{}] unknown message type 0x{:x}, flags {}: {:?}", peer_addr, o, f, b);
                if o == 0x00E5 {
                    Message::BbCharAck(0, BbCharAck { slot: 1, code: 0 }).serialize(&mut w_s).unwrap();
                }
            },
            Message::Login(_, Login { username, password, .. }) => {
                match check_login(ctx.db_pool.clone(), &username, &password) {
                    CheckLogin::Success => {
                        info!("[{}] character: {} logged in successfully", peer_addr, username);
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 0,
                            tag: 0,
                            guildcard: 1000000,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                    },
                    _ => {
                        // they shouldn't be at this point, so we're gonna send an error
                        let r = Message::LargeMsg(0, LargeMsg("Something happened recently that invalidated your account,\nso you cannot proceed.".to_string()));
                        r.serialize(&mut w_s).unwrap();
                        let r = Message::BbSecurity(0, BbSecurity {
                            err_code: 4, // maintenance
                            tag: 0,
                            guildcard: 0,
                            team_id: 0,
                            security_data: BbSecurityData::default(),
                            caps: 0
                        });
                        r.serialize(&mut w_s).unwrap();
                        return
                    }
                }
            },
            Message::BbOptionRequest(_, _) => {
                info!("[{}] character: request options", peer_addr);
                let r = Message::BbOptionConfig(0, BbOptionConfig(BbTeamAndKeyData::default()));
                r.serialize(&mut w_s).unwrap();
            },
            Message::BbChecksum(_, BbChecksum(cs)) => {
                info!("[{}] character: client checksum is {}", peer_addr, cs);
                let r = Message::BbChecksumAck(0, BbChecksumAck(true));
                r.serialize(&mut w_s).unwrap();
            },
            Message::BbGuildRequest(_, _) => {
                use crc::crc32::checksum_ieee;
                info!("[{}] character: guild card request", peer_addr);
                let checksum = checksum_ieee(&vec![0u8; 54672][..]);
                let r = Message::BbGuildCardHdr(0, BbGuildCardHdr {
                    one: 1,
                    len: 54672,
                    checksum: checksum
                });
                r.serialize(&mut w_s).unwrap();
            },
            Message::BbParamHdrReq(_, _) => {
                //let r = Message::LargeMsg(0, LargeMsg("Whoops, param files aren't ready yet.".to_string()));
                //r.serialize(&mut w_s).unwrap();
                param_chunks.0.serialize(&mut w_s).unwrap();
            },
            Message::BbParamChunkReq(chunk, _) => {
                // let r = Message::BbParamChunk(0, BbParamChunk { chunk: chunk, data: Vec::new() });
                // r.serialize(&mut w_s).unwrap();
                if let Some(ref a) = param_chunks.1.get(chunk as usize) {
                    a.serialize(&mut w_s).unwrap();
                } else {
                    Message::LargeMsg(0, LargeMsg("Whoops, you requested a chunk in the param table that doesn't exist.".to_string()))
                        .serialize(&mut w_s).unwrap();
                    return
                }
            },
            // 54672 or 0xD590 bytes of guild card nonsense
            Message::BbGuildCardChunkReq(_, BbGuildCardChunkReq(_, chunk, cont)) => {
                if cont {
                    let size_remaining: usize = 54672 - (chunk as usize * 0x6800);
                    let size: usize = if size_remaining < 0x6800 { size_remaining } else { 0x6800 };
                    info!("[{}] sending guild card chunk {} of size {}", peer_addr, chunk, size);
                    let r = Message::BbGuildCardChunk(0, BbGuildCardChunk {
                        unk: 0,
                        chunk: chunk,
                        data: vec![0u8; size]
                    });
                    r.serialize(&mut w_s).unwrap();
                }
            },
            Message::BbCharSelect(_, b) => {
                info!("[{}] bb char select {:?}", peer_addr, b);
                if b.selecting {
                    let r = Message::LargeMsg(0, LargeMsg("how'd you do that. you can't even make characters yet. get out.".to_string()));
                    r.serialize(&mut w_s).unwrap();
                    let r = Message::BbSecurity(0, BbSecurity {
                        err_code: 4,
                        tag: 0,
                        guildcard: 0,
                        team_id: 0,
                        security_data: BbSecurityData::default(),
                        caps: 0
                    });
                    r.serialize(&mut w_s).unwrap();
                    return
                } else {
                    let r = Message::BbCharAck(0, BbCharAck {
                        slot: b.slot,
                        code: 2 //nonexistant
                    });
                    r.serialize(&mut w_s).unwrap();
                }
            },
            Message::Goodbye(_, _) => {
                info!("[{}] character: goodbye", peer_addr);
                return
            },
            Message::BbSetFlags(_, BbSetFlags(f)) => {
                info!("[{}] character: set flag {}", peer_addr, f);
                return
            }
            a => info!("[{}] known but unconsidered message received: {:?}", peer_addr, a)
        }
    }
}

enum CheckLogin {
    Success,
    WrongPassword,
    Banned,
    NoAccount
}

fn check_login(db_pool: Arc<Pool>, username: &str, password: &str) -> CheckLogin {
    // TODO this really should be cleaned up
    let backend = db_pool.get_connection().unwrap();
    let account;
    {
        // new scope to drop lock as quickly as possible
        let b = backend.lock().unwrap();
        account = b.get_account_by_username(&username).unwrap();
    }
    match account {
        Some(a) => {
            if a.banned {
                CheckLogin::Banned
            } else {
                if a.cmp_password(&password, "") {
                    CheckLogin::Success
                } else {
                    CheckLogin::WrongPassword // should probably send NoAccount...
                }
            }
        },
        None => {
            CheckLogin::NoAccount
        }
    }
}

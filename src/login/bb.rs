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

pub struct Context {
    stream: TcpStream,
    key_table: Arc<Vec<u32>>,
    db_pool: Arc<Pool>
}

impl Context {
    pub fn new(stream: TcpStream, key_table: Arc<Vec<u32>>, db_pool: Arc<Pool>) -> Context {
        Context {
            stream: stream,
            key_table: key_table,
            db_pool: db_pool
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
            },
            Message::Login(_, Login { username, password, .. }) => {
                match check_login(ctx.db_pool.clone(), &username, &password) {
                    _ => {
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

// let r = Message::BbSecurity(0, BbSecurity {
//     err_code: 8, // user doesn't exist
//     tag: 0,
//     guildcard: 0,
//     team_id: 0,
//     security_data: vec![0u8; 40],
//     caps: 0
// });
// r.serialize(&mut w_s).unwrap();
// return Ok(())

// if a.banned {
//     let r = Message::BbSecurity(0, BbSecurity {
//         err_code: 6, // banned
//         tag: 0,
//         guildcard: 0,
//         team_id: 0,
//         security_data: vec![0u8; 40],
//         caps: 0
//     });
//     r.serialize(&mut w_s).unwrap();
//     return Ok(())
// } else {
//     // Check password
//     // TODO real salt
//     if a.cmp_password(&password, "") {

//         r.serialize(&mut w_s).unwrap();
//         return Ok(())
//     } else {
//         // password is invalid
//         let r = Message::BbSecurity(0, BbSecurity {
//             err_code: 2, // bad pw
//             tag: 0,
//             guildcard: 0,
//             team_id: 0,
//             security_data: vec![0u8; 40],
//             caps: 0
//         });
//         r.serialize(&mut w_s).unwrap();
//         return Ok(())
//     }
// }

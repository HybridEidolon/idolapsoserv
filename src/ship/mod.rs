//! Ship and block server.

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::net::{TcpListener, TcpStream};
use std::thread;

use ::db::Pool;

use psocrypto::{EncryptWriter, DecryptReader};
use psocrypto::bb::BbCipher;

mod error;

pub use self::error::{Result, Error, ErrorKind};

/// Messages bound for the central Ship.
enum ShipMsg {
    NewClient(TcpStream),
    DelClient,
    AcceptorClosing
}

/// Messages bound for the Client.
enum ClientMsg {
    LargeMsg(String),
    JoinBlockLobby(u8, u8),
    AbruptDisconnect
}

pub struct ShipServer {
    running: bool,
    clients: Vec<Arc<Client>>,
    bind: String,
    db_pool: Arc<Pool>,
    key_table: Arc<Vec<u32>>
}

pub struct Client {
    tx: Sender<ClientMsg>
}

impl ShipServer {
    pub fn new(bind: &str, key_table: Arc<Vec<u32>>, db_pool: Arc<Pool>) -> Self {
        ShipServer {
            running: true,
            clients: Vec::new(),
            bind: bind.to_owned(),
            db_pool: db_pool,
            key_table: key_table
        }
    }

    pub fn run(self) -> Result<()> {
        let ShipServer {
            mut running,
            mut clients,
            bind,
            db_pool,
            key_table
        } = self;
        // Set up the comms channel for all connections.

        let (tx, rx) = mpsc::channel();

        // Open a thread that accepts connections for us and adds to the client list, and a copy
        // of the transmitter to it.
        {
            let bind_c = bind.clone();
            let tx_c = tx.clone();
            thread::spawn(move|| client_acceptor(&bind_c, tx_c));
        }

        while running {
            use self::ShipMsg::*;

            let msg = rx.recv().unwrap();
            match msg {
                NewClient(mut s) => {
                    let peer_addr = match s.peer_addr() {
                        Err(_) => {error!("Failed to get peer address. Wow, I'm surprised we failed this early."); continue},
                        Ok(p) => p
                    };
                    match handle_newclient(&mut s, key_table.clone(), db_pool.clone(), tx.clone()) {
                        Err(e) => warn!("[{}] client failed to connect: {}", peer_addr, e),
                        Ok(c_tx) => {
                            let client = Client {
                                tx: c_tx
                            };
                            clients.push(Arc::new(client));
                        }
                    }
                },
                DelClient => {
                    info!("A client disconnected.");
                }
                AcceptorClosing => {
                    running = false;
                    for c in clients.iter() {
                        if let Err(_) = c.tx.send(ClientMsg::LargeMsg("Server is going offline due to client acceptor thread dying.".to_string())) {
                            continue
                        }
                        if let Err(_) = c.tx.send(ClientMsg::AbruptDisconnect) {
                            continue
                        }
                    }
                    continue
                }
            }
        }

        Ok(())
    }
}

fn client_acceptor(bind: &str, tx: Sender<ShipMsg>) {
    let tcp_listener = TcpListener::bind(bind).unwrap();

    for s in tcp_listener.incoming() {
        use self::ShipMsg::*;
        match s {
            Ok(stream) => match tx.send(ShipMsg::NewClient(stream)) {
                Err(_) => {tx.send(AcceptorClosing).unwrap(); return},
                _ => ()
            },
            _ => break
        }
    }

    tx.send(ShipMsg::AcceptorClosing).unwrap();
}

fn client_thread(mut w: EncryptWriter<TcpStream, BbCipher>, mut r: DecryptReader<TcpStream, BbCipher>, rx: Receiver<ClientMsg>, tx: Sender<ShipMsg>) {
    use psomsg::bb::*;
    use psomsg::Serial;
    use std::fs::File;
    use ::game::CharClass;

    let mut fc;

    info!("client connected to the ship server");

    {
        {
            let mut ll: Vec<(u32, u32)> = Vec::new();
            ll.push((60, 1));
            ll.push((60, 2));
            ll.push((60, 3));
            ll.push((60, 4));
            ll.push((60, 5));
            ll.push((60, 6));
            ll.push((60, 7));
            ll.push((60, 8));
            ll.push((60, 9));
            ll.push((60, 10));
            ll.push((60, 11));
            ll.push((60, 12));
            ll.push((60, 13));
            ll.push((60, 14));
            ll.push((60, 15));
            ll.push((0, 0));
            Message::LobbyList(16, LobbyList { items: ll }).serialize(&mut w).unwrap();
        }
        fc = ::util::nsc::read_nsc(&mut File::open("data/default/default_0.nsc").unwrap(), CharClass::HUmar).unwrap();
        fc.name = "\u{0009}Eguaco".to_string();
        fc.chara.name = "\u{0009}Eguaco".to_string();
        fc.guildcard = 1000000;
        fc.key_config.guildcard = 1000000;
        fc.chara.level = 30;
        fc.chara.hp = 400;
        Message::BbFullChar(0, BbFullChar(fc.clone())).serialize(&mut w).unwrap();
        Message::CharDataRequest(0, CharDataRequest).serialize(&mut w).unwrap();
    }

    loop {
        //use self::ClientMsg::*;
        let msg = Message::deserialize(&mut r);
        if let Err(_) = msg {
            if let Err(_) = tx.send(ShipMsg::DelClient) {
                error!("apparently the whole ship died...");
            }
            return
        } else if let Ok(msg) = msg { match msg {
            Message::BbCharDat(_, BbCharDat(data)) => {
                info!("{}", data.chara.name);
                let mut l = LobbyJoin::default();
                l.client_id = 0;
                l.leader_id = 0;
                l.one = 1;
                l.lobby_num = 0;
                l.block_num = 1;
                l.event = 0;
                let mut lm = LobbyMember::default();
                lm.hdr.guildcard = 1000000;
                lm.hdr.tag = 0x00010000;
                lm.hdr.client_id = 0;
                lm.hdr.name = fc.name.clone();
                lm.data.name = fc.chara.name.clone();
                lm.data.name_color = 0xFFFFFFFF;
                lm.data.section = 1;
                lm.data.class = 1;
                lm.data.level = 30;
                lm.data.version = 3;
                lm.data.v1flags = 25;
                lm.data.hp = 400;
                lm.data.model = 0;
                lm.data.skin = 1;
                lm.data.face = 1;
                lm.data.head = 1;
                lm.data.hair = 1;
                lm.data.prop_x = 1.0;
                lm.data.prop_y = 1.0;
                //l.members.push(lm);
                Message::LobbyJoin(0, l).serialize(&mut w).unwrap();
                Message::LobbyArrowList(0, LobbyArrowList(Vec::new())).serialize(&mut w).unwrap();
            }
            a => info!("block client recv msg {:?}", a)
        }}
    }
}

/// Spawn a new client thread that receives messages and channels them to the ship server for
/// further handling.
fn handle_newclient(stream: &mut TcpStream, key_table: Arc<Vec<u32>>, db_pool: Arc<Pool>, tx: Sender<ShipMsg>) -> Result<Sender<ClientMsg>> {
    use psomsg::bb::*;
    use psomsg::Serial;
    use rand::random;
    use ::db::Account;

    // Generate our crypto.
    let s_key = vec![random::<u8>(); 48];
    let c_key = vec![random::<u8>(); 48];
    let s_cipher = BbCipher::new(&s_key, &key_table);
    let c_cipher = BbCipher::new(&c_key, &key_table);

    // Send the welcome
    try!(Message::BbWelcome(0, BbWelcome(s_key, c_key)).serialize(stream));

    // Set up crypto streams
    let mut w_s = EncryptWriter::new(try!(stream.try_clone()), s_cipher);
    let mut r_s = DecryptReader::new(try!(stream.try_clone()), c_cipher);

    // Wait for login
    match Message::deserialize(&mut r_s) {
        Ok(Message::BbLogin(0, BbLogin { username, password, security_data, .. })) => {
            // Verify credentials with database
            let conn = match db_pool.get_connection() {
                Ok(c) => c,
                Err(e) => return Err(Error::new(ErrorKind::DbError, e))
            };

            let account: Account;
            {
                let db = match conn.lock() {
                    Ok(d) => d,
                    Err(e) => return Err(Error::new(ErrorKind::DbError, "Poisoned connection lock!"))
                };

                account = match db.get_account_by_username(&username) {
                    Ok(None) => return Err(Error::new(ErrorKind::CredentialError, format!("User with name {} doesn't exist.", username))),
                    Ok(Some(a)) => a,
                    Err(e) => return Err(Error::new(ErrorKind::DbError, e))
                };
            }

            if !account.cmp_password(&password, "") {
                if let Err(e) = Message::BbSecurity(0, BbSecurity {
                    err_code: 4,
                    tag: 0,
                    guildcard: 0,
                    team_id: 0,
                    security_data: security_data,
                    caps: 0
                }).serialize(&mut w_s) {
                    // do nothing, we're gonna drop them anyway
                }
                return Err(Error::new(ErrorKind::CredentialError, "Incorrect password."))
            }

            if account.banned {
                if let Err(e) = Message::BbSecurity(0, BbSecurity {
                    err_code: 7,
                    tag: 0,
                    guildcard: 0,
                    team_id: 0,
                    security_data: security_data,
                    caps: 0
                }).serialize(&mut w_s) {
                    // do nothing, we're gonna drop them anyway
                }
                return Err(Error::new(ErrorKind::CredentialError, "User is banned."))
            }
            if let Err(e) = Message::BbSecurity(0, BbSecurity {
                err_code: 0,
                tag: 0x00010000,
                guildcard: 1000000,
                team_id: 1,
                security_data: security_data,
                caps: 0x00000102
            }).serialize(&mut w_s) {
                return Err(Error::new(ErrorKind::IoError, e))
            }
        },
        Ok(_) => return Err(Error::new(ErrorKind::UnexpectedMsg, "User was expected to BbLogin but did not.")),
        Err(e) => return Err(Error::new(ErrorKind::IoError, e))
    }

    // New transmission channel between client handler thread and the ship/block server.
    let (tx_c, rx) = mpsc::channel();

    thread::spawn(move|| client_thread(w_s, r_s, rx, tx));

    Ok(tx_c)
}

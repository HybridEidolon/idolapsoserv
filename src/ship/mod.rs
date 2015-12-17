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
            mut bind,
            mut db_pool,
            mut key_table
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
                        Err(e) => {error!("Failed to get peer address. Wow, I'm surprised we failed this early."); continue},
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
                        if let Err(e) = c.tx.send(ClientMsg::LargeMsg("Server is going offline due to client acceptor thread dying.".to_string())) {
                            continue
                        }
                        if let Err(e) = c.tx.send(ClientMsg::AbruptDisconnect) {
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

    info!("client connected to the ship server");

    {
        let mut l = LobbyJoin::default();
        l.client_id = 4;
        l.leader_id = 4;
        l.lobby_num = 0;
        l.block_num = 1;
        l.event = 0;
        Message::LobbyJoin(0, l).serialize(&mut w).unwrap();
        let mut l = LobbyAddMember::default();
        l.client_id = 4;
        l.leader_id = 4;
        l.lobby_num = 0;
        l.block_num = 1;
        l.event = 0;
        let mut lm = LobbyMember::default();
        lm.hdr.guildcard = 1000000;
        lm.hdr.tag = 0x00010000;
        lm.hdr.client_id = 4;
        lm.hdr.name = "\tguaco".to_string();
        lm.data.level = 0;
        lm.data.name = "\tguaco".to_string();
        l.members.push(lm);
        Message::LobbyAddMember(1, l).serialize(&mut w).unwrap();
    }

    loop {
        use self::ClientMsg::*;
        let msg = Message::deserialize(&mut r);
        if let Err(ref e) = msg {
            if let Err(se) = tx.send(ShipMsg::DelClient) {
                error!("apparently the whole ship died..."); return
            }
            return
        } else if let Ok(msg) = msg { match msg {
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

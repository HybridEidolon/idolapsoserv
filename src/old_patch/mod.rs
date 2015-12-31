//! Structures for the Patch and Data servers.

use std::net::{TcpListener, TcpStream, SocketAddrV4};
use std::io;
use std::borrow::Borrow;
use std::thread;
use std::string::ToString;

use psocrypto::pc::PcCipher;
use psocrypto::{DecryptReader, EncryptWriter};

use psomsg::Serial;

use rand::random;

pub struct PatchServer {
    motd_template: String,
    bind: String,
    data_servers: Vec<SocketAddrV4>
}

pub struct DataServer {
    bind: String
}

pub struct ClientContext {
    stream: TcpStream,
    motd: String,
    data_addr: Option<SocketAddrV4>
}

impl ClientContext {
    pub fn run(&mut self) -> () {
        use psomsg::patch::*;

        let peer_addr = self.stream.peer_addr().unwrap();

        info!("[{}] connected", peer_addr);

        let server_cipher = PcCipher::new(random());
        let client_cipher = PcCipher::new(random());

        let w = Message::Welcome(Some(Welcome {
            server_vector: server_cipher.seed(),
            client_vector: client_cipher.seed()
        }));
        if let Err(e) = w.serialize(&mut self.stream) {
            error!("unable to send welcome message to {}: {}", peer_addr, e);
        }
        info!("[{}] welcomed", peer_addr);

        // now wrap our stream in crypto
        let mut w_s = EncryptWriter::new(self.stream.try_clone().unwrap(), server_cipher);
        let mut r_s = DecryptReader::new(self.stream.try_clone().unwrap(), client_cipher);

        // Client will send a Welcome message as an ack. We reply with a Login ack.
        if let Ok(s) = Message::deserialize(&mut r_s) {
            if let Message::Welcome(None) = s {
                info!("[{}] responded to welcome", peer_addr);
                match Message::Login(None).serialize(&mut w_s) { Err(_) => return, _ => () }
            }
        }

        loop {
            // Read message
            if let Ok(s) = Message::deserialize(&mut r_s) {match s {
                Message::Login(Some(_)) => {
                    info!("[{}] logged in, sending motd and redirecting", peer_addr);
                    let motd = Message::Motd(Some(Motd {
                        message: self.motd.clone()
                    }));
                    motd.serialize(&mut w_s).unwrap();
                    let red = Message::Redirect(Some(Redirect(self.data_addr.as_ref().unwrap().clone())));
                    red.serialize(&mut w_s).unwrap();
                    // Now we break out of this connection.
                    return;
                },
                u => {error!("unexpected message received, exiting: {:?}", u); return}
            }}
        };
    }

    pub fn run_data(&mut self) -> () {
        use psomsg::patch::*;
        use staticvec::StaticVec;
        let peer = self.stream.peer_addr().unwrap();

        info!("connected {}", peer);

        let server_cipher = PcCipher::new(random());
        let client_cipher = PcCipher::new(random());

        let w = Message::Welcome(Some(Welcome {
            client_vector: client_cipher.seed(),
            server_vector: server_cipher.seed()
        }));
        w.serialize(&mut self.stream).unwrap();

        // now wrap our stream in crypto
        let mut w_s = EncryptWriter::new(self.stream.try_clone().unwrap(), server_cipher);
        let mut r_s = DecryptReader::new(self.stream.try_clone().unwrap(), client_cipher);

        if let Ok(Message::Welcome(None)) = Message::deserialize(&mut r_s) {
            Message::Login(None).serialize(&mut w_s).unwrap();
        }

        loop {
            let m = Message::deserialize(&mut r_s);
            if let Ok(m) = m {match m {
                Message::Login(Some(_)) => {
                    Message::StartList(None).serialize(&mut w_s).unwrap();
                    Message::SetDirectory(Some(SetDirectory { dirname: StaticVec::default() })).serialize(&mut w_s).unwrap();
                    Message::InfoFinished(None).serialize(&mut w_s).unwrap();
                },
                Message::FileListDone(_) => {
                    Message::SetDirectory(Some(SetDirectory { dirname: StaticVec::default() })).serialize(&mut w_s).unwrap();
                    Message::OneDirUp(None).serialize(&mut w_s).unwrap();
                    Message::SendDone(None).serialize(&mut w_s).unwrap();
                    info!("client {} was 'updated' successfully", peer);
                }
                e => error!("recv something else! {:?}", e)
            }} else {
                return
            }
        }
    }
}

unsafe impl Send for ClientContext {}

impl PatchServer {
    pub fn new_bb<T: ToString, B: ToString>(motd_template: T, bind: B, data_servers: &[SocketAddrV4]) -> PatchServer {
        PatchServer {
            motd_template: motd_template.to_string(),
            bind: bind.to_string(),
            data_servers: data_servers.to_owned()
        }
    }

    pub fn format_motd(&self, client_num: u32) -> String {
        let motd = self.motd_template.replace("{client_num}", &format!("{}", client_num));
        motd
    }

    /// Runs the patch server, moving this value. This method will block until the server
    /// concludes.
    pub fn run(self) -> io::Result<()> {
        let bind_addr = self.bind.clone(); // borrow checker just in case
        let tcp_listener = try!(TcpListener::bind(bind_addr.borrow() as &str));

        let mut total_connects = 0;

        for s in tcp_listener.incoming() {
            match s {
                Ok(s) => {
                    total_connects += 1;
                    let mut ctx = ClientContext {
                        stream: s,
                        motd: self.format_motd(total_connects),
                        data_addr: Some(self.data_servers[random::<usize>() % self.data_servers.len()])
                    };

                    thread::spawn(move || {ctx.run();});
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }
        Ok(())
    }
}

impl DataServer {
    pub fn new_bb<B: ToString>(bind: B) -> DataServer {
        DataServer {
            bind: bind.to_string()
        }
    }

    pub fn run(self) -> io::Result<()> {
        let bind_addr = self.bind.clone(); // borrow checker just in case
        let tcp_listener = try!(TcpListener::bind(bind_addr.borrow() as &str));

        for s in tcp_listener.incoming() {
            match s {
                Ok(s) => {
                    let mut ctx = ClientContext {
                        stream: s,
                        motd: "".to_string(),
                        data_addr: None
                    };

                    thread::spawn(move || {ctx.run_data();});
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }
        Ok(())
    }
}

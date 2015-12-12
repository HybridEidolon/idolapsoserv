//! Structures for the Patch and Data servers.

use std::net::{TcpListener, TcpStream, SocketAddrV4};
use std::io;
use std::io::{Read, Write};
use std::borrow::Borrow;
use std::thread;
use std::string::ToString;

use psocrypto::pc::PcCipher;
use psocrypto::{Encryptor, Decryptor};

use rand::random;

use ::context::Context;

pub struct PatchServer {
    motd_template: String,
    bind: String,
    data_servers: Vec<SocketAddrV4>
}

pub struct DataServer {
    bind: String
}

pub struct ClientContext {
    server_cipher: PcCipher,
    client_cipher: PcCipher,
    stream: TcpStream,
    motd: String,
    data_addr: Option<SocketAddrV4>
}

impl Context for ClientContext {
    #[inline]
    fn get_write_encryptor(&mut self) -> io::Result<(&mut Write, &mut Encryptor)> {
        Ok((&mut self.stream, &mut self.server_cipher))
    }
    #[inline]
    fn get_read_decryptor(&mut self) -> io::Result<(&mut Read, &mut Decryptor)> {
        Ok((&mut self.stream, &mut self.client_cipher))
    }
}

impl ClientContext {
    pub fn run(&mut self) -> () {
        use psomsg::patch::*;

        let peer_addr = self.stream.peer_addr().unwrap();

        info!("client {} connected", peer_addr);

        let w = Welcome {
            server_vector: self.server_cipher.seed(),
            client_vector: self.client_cipher.seed()
        };
        if let Err(e) = self.send_msg_unenc(&w) {
            error!("unable to send welcome message to {}: {}", peer_addr, e);
            return
        }

        // Client will send a Welcome message as an ack. We reply with a Login ack.
        if let Ok(s) = self.recv_msg() {
            if let Message::Welcome(None) = s {
                match self.send_msg(&Message::Login(None)) { Err(_) => return, _ => () }
            }
        }

        loop {
            // Read message
            if let Ok(s) = self.recv_msg() {match s {
                Message::Login(Some(_)) => {
                    let motd = Motd {
                        message: self.motd.clone()
                    };
                    self.send_msg(&motd).unwrap();
                    let red = Redirect { ip_addr: self.data_addr.as_ref().unwrap().ip().clone(), port: self.data_addr.as_ref().unwrap().port() };
                    self.send_msg(&red).unwrap();
                    // Now we break out of this connection.
                    return;
                },
                u => {error!("unexpected message received, exiting: {:?}", u); return}
            }}
        };
    }

    pub fn run_data(&mut self) -> () {
        use psomsg::patch::*;
        use psomsg::staticvec::StaticVec;
        let peer = self.stream.peer_addr().unwrap();

        info!("connected {}", peer);

        let w = Welcome {
            client_vector: self.client_cipher.seed(),
            server_vector: self.server_cipher.seed()
        };
        self.send_msg_unenc(&w).unwrap();

        if let Ok(Message::Welcome(None)) = self.recv_msg() {
            self.send_msg(&Message::Login(None)).unwrap();
        }

        loop {
            let m = self.recv_msg();
            if let Ok(m) = m {match m {
                Message::Login(Some(_)) => {
                    self.send_msg(&StartList).unwrap();
                    self.send_msg(&SetDirectory { dirname: StaticVec::default() }).unwrap();
                    self.send_msg(&InfoFinished).unwrap();
                    //ctx.send_msg(&FileListDone, true).unwrap();
                },
                Message::FileListDone(_) => {
                    self.send_msg(&SetDirectory { dirname: StaticVec::default() }).unwrap();
                    self.send_msg(&OneDirUp).unwrap();
                    self.send_msg(&SendDone).unwrap();
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
                        server_cipher: PcCipher::new(random()),
                        client_cipher: PcCipher::new(random()),
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
                        server_cipher: PcCipher::new(random()),
                        client_cipher: PcCipher::new(random()),
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

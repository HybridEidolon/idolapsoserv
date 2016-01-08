extern crate psocrypto;
extern crate psomsg;
extern crate psoserial;
extern crate docopt;
extern crate rustc_serialize;
extern crate byteorder;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate toml;

use docopt::Docopt;

use std::io::{Read, Cursor};
use std::io;

use std::net::{Ipv4Addr, TcpListener, TcpStream, SocketAddrV4};

use std::fs::File;

use std::sync::Arc;

use std::thread;
use std::net::Shutdown;
use std::time::Duration;

pub mod util;
mod config;
use util::*;
use config::read_config;

use psomsg::util::*;

static USAGE: &'static str = "
PSOBB Proxy

Usage:
  bbproxy [options]
  bbproxy (-h | --help)

Options:
  -h,--help             Show this message.
  --config=<config>     Path to config [Default: bbproxy.toml].

";

#[derive(RustcDecodable, Clone, Debug)]
struct Args {
    flag_config: String
}

fn read_key_table(r: &mut Read) -> io::Result<Vec<u32>> {
    let mut data = Vec::with_capacity(1042 * 4);
    try!(r.read_to_end(&mut data));
    let mut key_table: Vec<u32> = Vec::with_capacity(1042);
    let mut cur = Cursor::new(data);
    loop {
        use byteorder::{LittleEndian, ReadBytesExt};
        match cur.read_u32::<LittleEndian>() {
            Err(_) => break,
            Ok(n) => key_table.push(n)
        }
    }
    Ok(key_table)
}

macro_rules! fileline {
    () => {
        format!("{}:{}", file!(), line!())
    }
}

fn spawn_pc_proxy_receiver_thread(ip: Ipv4Addr, port: u16, local_port: u16, persistent: bool) {
    use std::thread;
    let tcp = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), local_port)).unwrap();
    println!("Opened PC proxy socket to {}:{} on port {}", ip, port, local_port);
    thread::spawn(move|| {
        match tcp.accept() {
            Ok((s, _)) => {
                println!("New PC client");
                let ip_c = ip.clone();
                thread::spawn(move|| pc_proxy_thread(s, ip_c, port));
                if !persistent {
                    debug!("Non-persistent PC listener dying off");
                    return
                }
            },
            Err(_) => return
        }
    });
}

fn pc_proxy_thread(mut client_stream: TcpStream, ip: Ipv4Addr, port: u16) {
    use psomsg::Serial;
    use psomsg::patch::*;
    use psocrypto::pc::PcCipher;
    use psocrypto::{EncryptWriter, DecryptReader};

    println!("Connecting patch proxy to {}:{}", ip, port);
    let mut server_stream = match TcpStream::connect((ip, port)) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect: {}", e);
            return
        }
    };

    let mut s_w;
    let mut s_r;
    let mut c_w;
    let mut c_r;

    // Receive "Welcome" from server
    if let Ok(Message::Welcome(Some(Welcome { client_vector, server_vector }))) = Message::deserialize(&mut server_stream) {
        let from_server = PcCipher::new(server_vector);
        let to_server = PcCipher::new(client_vector);
        let from_client = PcCipher::new(0);
        let to_client = PcCipher::new(0);

        s_w = EncryptWriter::new(server_stream.try_clone().unwrap(), to_server);
        s_r = DecryptReader::new(server_stream.try_clone().unwrap(), from_server);
        c_w = EncryptWriter::new(client_stream.try_clone().unwrap(), to_client);
        c_r = DecryptReader::new(client_stream.try_clone().unwrap(), from_client);

        println!("Welcomed a PC client");
        Message::Welcome(Some(Welcome { client_vector: 0, server_vector: 0 })).serialize(&mut client_stream).unwrap();
    } else {
        println!("Failed to welcome PC");
        return
    }

    // Now we create two threads, one for send, one for receive.
    thread::spawn(move|| {
        loop {
            let msg = match Message::deserialize(&mut s_r) {
                Ok(m) => m,
                Err(e) => {
                    error!("Server reading error: {}\n at {}", e, fileline!());
                    s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                    c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                    return
                }
            };
            //println!("S-->C\n{}", hex_view_serial(&msg));
            if let &Message::Redirect(Some(Redirect(ref socket))) = &msg {
                // Spawn a new proxy receiver and change the redirect.
                let new_ip = socket.ip().clone();
                let new_port = socket.port() + 20000;
                info!("Redirecting local client to {}:{}", new_ip, new_port);
                spawn_pc_proxy_receiver_thread(new_ip, new_port - 20000, new_port, false);
                thread::sleep(Duration::from_millis(100));
                Message::Redirect(Some(Redirect(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), new_port)))).serialize(&mut c_w).unwrap();

                println!("Redirected, shutting down this thread");
                thread::sleep(Duration::from_millis(100));
                s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                return
            } else {
                match msg.serialize(&mut c_w) {
                    Err(e) => {
                        error!("Client writing error: {}\n at {}", e, fileline!());
                        s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                        c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                        return
                    },
                    _ => ()
                }
            }
        }
    });
    thread::spawn(move|| {
        loop {
            let msg = match Message::deserialize(&mut c_r) {
                Ok(m) => m,
                Err(e) => {
                    error!("Client reading error: {}\n at {}", e, fileline!());
                    c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                    s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                    return
                }
            };
            //println!("C-->S\n{}", hex_view_serial(&msg));
            match msg.serialize(&mut s_w) {
                Err(e) => {
                    error!("Server writing error: {}\n at {}", e, fileline!());
                    c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                    s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                    return
                },
                _ => ()
            }
        }
    });
}

fn spawn_bb_proxy_receiver_thread(ip: Ipv4Addr, port: u16, local_port: u16, client_keytable: Arc<Vec<u32>>, server_keytable: Arc<Vec<u32>>, persistent: bool) {
    debug!("Opening BB proxy socket to {}:{} on port {}", ip, port, local_port);
    let tcp = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), local_port)).unwrap();
    thread::spawn(move|| {
        match tcp.accept() {
            Ok((s, _)) => {
                let ip_c = ip.clone();
                let ck = client_keytable.clone();
                let sk = server_keytable.clone();
                println!("New BB client");
                thread::spawn(move|| bb_proxy_thread(s, ip_c, port, ck, sk));
                if !persistent {
                    debug!("Non-persistent BB listener dying off");
                    return
                }
            },
            Err(_) => return
        }
    });
}

fn bb_proxy_thread(mut client_stream: TcpStream, ip: Ipv4Addr, port: u16, client_keytable: Arc<Vec<u32>>, server_keytable: Arc<Vec<u32>>) {
    use psomsg::Serial;
    use psomsg::bb::*;
    use psocrypto::bb::BbCipher;
    use psocrypto::{EncryptWriter, DecryptReader};

    let mut server_stream = TcpStream::connect((ip, port)).unwrap();

    let mut s_w;
    let mut s_r;
    let mut c_w;
    let mut c_r;

    // Receive "Welcome" from server
    if let Ok(Message::BbWelcome(_, BbWelcome(server_vector, client_vector))) = Message::deserialize(&mut server_stream) {
        let server_server_cipher;
        let server_client_cipher;

        let client_server_cipher;
        let client_client_cipher;

        server_server_cipher = BbCipher::new(&server_vector, &server_keytable);
        server_client_cipher = BbCipher::new(&client_vector, &server_keytable);
        client_server_cipher = BbCipher::new(&vec![0; 48], &client_keytable);
        client_client_cipher = BbCipher::new(&vec![0; 48], &client_keytable);

        s_w = EncryptWriter::new(server_stream.try_clone().unwrap(), server_client_cipher);
        s_r = DecryptReader::new(server_stream.try_clone().unwrap(), server_server_cipher);
        c_w = EncryptWriter::new(client_stream.try_clone().unwrap(), client_server_cipher);
        c_r = DecryptReader::new(client_stream.try_clone().unwrap(), client_client_cipher);

        println!("Welcomed a BB client");
        Message::BbWelcome(0, BbWelcome(vec![0; 48], vec![0; 48])).serialize(&mut client_stream).unwrap();
    } else {
        return
    }

    // Now we create two threads, one for send, one for receive.
    let ck = client_keytable.clone();
    let sk = server_keytable.clone();
    thread::spawn(move|| {
        loop {
            let buf = match read_bb_msg(&mut s_r) {
                Ok(b) => b,
                Err(e) => {
                    error!("Server reading error: {}\n at {}", e, fileline!());
                    // s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                    // c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                    return
                }
            };
            let msg = match Message::deserialize(&mut Cursor::new(&buf)) {
                Ok(m) => m,
                Err(e) => {
                    use std::io::Write;
                    error!("Server read serialization failure: {}\n at {}", e, fileline!());
                    error!("Unparsed buffer\n{}", hex_view(&buf));
                    error!("Continuing by sending raw buffer");
                    match c_w.write_all(&buf) {
                        Err(e) => {
                            error!("Client writing error: {}\n at {}", e, fileline!());
                            // s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                            // c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                            return
                        },
                        _ => continue
                    }
                }
            };
            let bad_serial;
            {
                let mut cursor = Cursor::new(Vec::new());
                msg.serialize(&mut cursor).unwrap();
                let buf2 = cursor.into_inner();
                if buf != buf2 {
                    warn!("The read buffer and the parsed result are not identical!");
                    warn!("Serialized\n{}", hex_view_diff(&msg, &buf));
                    warn!("S-->C\n{}", hex_view(&buf));
                    bad_serial = true;
                } else {
                    bad_serial = false;
                }
            }
            println!("S-->C\n{}", hex_view(&buf));
            if let &Message::Redirect(0, Redirect { ref ip, ref port }) = &msg {
                use std::net::Shutdown;
                use std::time::Duration;
                // Spawn a new proxy receiver and change the redirect.
                let new_port = *port + 20000;
                spawn_bb_proxy_receiver_thread(ip.clone(), *port, new_port, ck, sk, false);
                thread::sleep(Duration::from_millis(100));
                Message::Redirect(0, Redirect { ip: Ipv4Addr::new(127, 0, 0, 1), port: new_port } ).serialize(&mut c_w).unwrap();

                thread::sleep(Duration::from_millis(100));
                s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                return
            } else {
                if let &Message::BbFullChar(_, _) = &msg {
                    info!("Saving character information.");
                    {
                        let mut file = File::create("../char_serial.bin").unwrap();
                        msg.serialize(&mut file).unwrap();
                    }
                    {
                        use std::io::Write;
                        let mut file = File::create("../char_raw.bin").unwrap();
                        file.write_all(&buf).unwrap();
                    }
                }
                if !bad_serial {
                    match msg.serialize(&mut c_w) {
                        Err(e) => {
                            error!("Client writing error: {}\n at {}", e, fileline!());
                            // s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                            // c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                            return
                        },
                        _ => ()
                    }
                } else {
                    use std::io::Write;
                    match c_w.write_all(&buf) {
                        Err(e) => {
                            error!("Client writing error: {}\n at {}", e, fileline!());
                            // s_r.into_inner().shutdown(Shutdown::Both).unwrap();
                            // c_w.into_inner().shutdown(Shutdown::Both).unwrap();
                            return
                        },
                        _ => ()
                    }
                }
            }
        }
    });
    let ck = client_keytable.clone();
    let sk = server_keytable.clone();
    thread::spawn(move|| {
        loop {
            let buf = match read_bb_msg(&mut c_r) {
                Ok(b) => b,
                Err(e) => {
                    error!("Client reading error: {}\n at {}", e, fileline!());
                    // c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                    // s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                    return
                }
            };
            let msg = match Message::deserialize(&mut Cursor::new(&buf)) {
                Ok(m) => m,
                Err(e) => {
                    use std::io::Write;
                    error!("Client read serialization failure: {}\n at {}", e, fileline!());
                    error!("Unparsed buffer\n{}", hex_view(&buf));
                    error!("Continuing by sending raw buffer");
                    match s_w.write_all(&buf) {
                        Err(e) => {
                            error!("Server writing error: {}\n at {}", e, fileline!());
                            c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                            s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                            return
                        },
                        _ => continue
                    }
                }
            };
            let bad_serial;
            {
                let mut cursor = Cursor::new(Vec::new());
                msg.serialize(&mut cursor).unwrap();
                let buf2 = cursor.into_inner();
                if buf != buf2 {
                    warn!("The read buffer and the parsed result are not identical!");
                    warn!("Serialized\n{}", hex_view_diff(&msg, &buf));
                    warn!("C-->S\n{}", hex_view(&buf));
                    bad_serial = true;
                } else {
                    bad_serial = false;
                }
            }
            println!("C-->S\n{}", hex_view(&buf));
            if !bad_serial {
                match msg.serialize(&mut s_w) {
                    Err(e) => {
                        error!("Server writing error: {}\n at {}", e, fileline!());
                        // c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                        // s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                        return
                    },
                    _ => ()
                }
                match msg {
                    Message::Goodbye(_, _) => {
                        use std::net::SocketAddr;
                        info!("They said goodbye, they PROBABLY intend to reconnect.");
                        let s_stream = s_w.into_inner();
                        let c_stream = c_r.into_inner();
                        let peer_addr = s_stream.peer_addr().unwrap();
                        let local_addr = c_stream.local_addr().unwrap();
                        if let SocketAddr::V4(v4sock) = peer_addr {
                            if let SocketAddr::V4(v4proxy) = local_addr {
                                c_stream.shutdown(Shutdown::Both).unwrap();
                                s_stream.shutdown(Shutdown::Both).unwrap();
                                debug!("Killing client proxy thread due to Goodbye");
                                debug!("Spawning reconnect receiver");
                                spawn_bb_proxy_receiver_thread(*v4sock.ip(), v4sock.port(), v4proxy.port(), ck, sk, false);
                            }
                        } else {
                            panic!("ipv6 not supported")
                        }
                        //thread::sleep(Duration::from_millis(100));
                        return
                    },
                    Message::BbSetFlags(_, _) => {
                        use std::net::SocketAddr;
                        info!("They said BbSetFlags, they DEFINITELY intend to reconnect.");
                        let s_stream = s_w.into_inner();
                        let c_stream = c_r.into_inner();
                        let peer_addr = s_stream.peer_addr().unwrap();
                        let local_addr = c_stream.local_addr().unwrap();
                        if let SocketAddr::V4(v4sock) = peer_addr {
                            if let SocketAddr::V4(v4proxy) = local_addr {
                                c_stream.shutdown(Shutdown::Both).unwrap();
                                s_stream.shutdown(Shutdown::Both).unwrap();
                                debug!("Killing client proxy thread due to BbSetFlags");
                                debug!("Spawning reconnect receiver");
                                spawn_bb_proxy_receiver_thread(*v4sock.ip(), v4sock.port(), v4proxy.port(), ck, sk, false);
                            }
                        } else {
                            panic!("ipv6 not supported")
                        }
                        //thread::sleep(Duration::from_millis(100));
                        return
                    },
                    _ => ()
                }
            } else {
                use std::io::Write;
                match s_w.write_all(&buf) {
                    Err(e) => {
                        error!("Server writing error: {}\n at {}", e, fileline!());
                        // c_r.into_inner().shutdown(Shutdown::Both).unwrap();
                        // s_w.into_inner().shutdown(Shutdown::Both).unwrap();
                        return
                    },
                    _ => ()
                }
            }
        }
    });
}

fn main() {
    use std::str::FromStr;
    use std::time::Duration;

    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let config = read_config(&args.flag_config).expect("Config file could not be read");
    println!("Press Ctrl-C to end the proxy.");

    let server_keytable = Arc::new(read_key_table(&mut File::open(&config.server_keytable).unwrap()).unwrap());
    let client_keytable = Arc::new(read_key_table(&mut File::open(&config.client_keytable).unwrap()).unwrap());

    spawn_pc_proxy_receiver_thread(FromStr::from_str(&config.server_patch_ip).unwrap(), config.server_patch_port, config.client_patch_port, true);
    spawn_bb_proxy_receiver_thread(FromStr::from_str(&config.server_login_ip).unwrap(), config.server_login_port, config.client_login_port, client_keytable.clone(), server_keytable.clone(), true);

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

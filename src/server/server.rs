use std::net::IpAddr;
use std::net::TcpStream;
use bincode::deserialize;
use std::net;
use std::thread;
use std::io::prelude::*;
use std::sync::{Mutex, Arc};
// use std::sync::mpsc;
use std::collections::HashMap;
use model::message;

pub struct Server {
    pub listener        : net::TcpListener,
    pub server_version  : Arc<u32>,
        peers           : Arc<Mutex<HashMap<IpAddr, TcpStream>>>,
        // sender          : mpsc::Sender<TcpStream>,
        // receiver        : mpsc::Receiver<TcpStream>,
}

impl Server {
    pub fn new() -> Server {
        // let (tx, rx) = mpsc::channel();
        Server {
            listener        : net::TcpListener::bind("0.0.0.0:4224").unwrap(),
            server_version  : Arc::new(1),
            peers           : Arc::new(Mutex::new(HashMap::new())),
            // sender          : tx,
            // receiver        : rx,
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap().try_clone().unwrap();
            let mut peers = self.peers.lock().unwrap();
            if !peers.contains_key(&stream.peer_addr().unwrap().ip()) {
                self.peers.lock().unwrap().insert(stream.peer_addr().unwrap().ip(), stream.try_clone().unwrap());
            }
            let peers = self.peers.clone();
            let mut connection_version = Arc::clone(&self.server_version);
            thread::Builder::new().name(stream.peer_addr().unwrap().ip().to_string()).spawn(move || {
                //read message
                let mut magic : [u8; 4] = [0; 4];
                let mut message_type : [u8; 12] = [0; 12];
                let mut length : [u8; 8] = [0; 8];
                let mut i_know_you : bool = false;

                loop {
                    stream.read(&mut magic).unwrap();
                    let mut magic = magic.to_vec();
                    magic.reverse();
                    let magic : u32 = deserialize(&magic).unwrap();
                    dbg!(&magic);
                    stream.read(&mut message_type).unwrap();
                    stream.read(&mut length).unwrap();

                    let mut length = length.to_vec();
                    length.reverse();
                    let size : u64 = deserialize(&length).unwrap();
                    let message_type : String = String::from_utf8(message_type.to_vec()).unwrap();
                    if !i_know_you {
                        match message_type.as_str() {
                            "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                                println!("Recieved message whoami");
                                connection_version = message::WhoAmI::read(&stream).handle(&stream, 1).unwrap();
                            },
                            "whoamiack\u{0}\u{0}\u{0}" => {
                                i_know_you = true;
                            },
                            _ => { println!("Recieved incorrect message_type : {:?}", message_type.as_str()); break; }
                        }

                    }
                    else {
                        if size > 0 {
                            match message_type.as_str() {
                                "getaddr" => {},
                                "addr" => {},
                                _ => ()
                            }
                        }
                    }
                }
                peers.lock().unwrap().remove(&stream.peer_addr().unwrap().ip());
            }).unwrap();
        }
    }

    pub fn start(&self) {
        let peers = self.peers.clone();
        thread::spawn(move || {
            let mut command : String = "".into();
            let mut ip : String = "".into();
            println!("Ensicoin started");
            loop {
                std::io::stdin().read_line(&mut command).unwrap();
                match command.as_ref() {
                    "connect\n" => {
                        println!("Enter a valid ip address: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        dbg!(&ip);
                        let peer = TcpStream::connect(std::net::SocketAddr::new(ip.parse().unwrap(), 4224));
                        match peer {
                            Ok(peer)    => {
                                let peers = peers.clone();
                                let mut peers = peers.lock().unwrap();

                                let ip = peer.peer_addr().unwrap().ip();
                                peers.insert(ip, peer);
                                let peer = peers.get_mut(&ip).unwrap();
                                let message = message::WhoAmI::new();
                                message::WhoAmI::send(message, &peer).unwrap();
                            },
                            Err(oops)   => println!("Could not connect to {} .\n Error: {:?}", &ip, oops)
                        }
                    }
                    _ => ()
                }
                command = "".into();
                ip = "".into();
            }
        });
    }
}

// fn say_hello(stream : &TcpStream) {
//     let stream = stream.try_clone().unwrap();
//     thread::spawn(move || {
//
//     });
// }

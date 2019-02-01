// use blockchain::blockchain::Blockchain;
// use model::block::Block;
use std::net::IpAddr;
use std::net::TcpStream;
use bincode::deserialize;
use model::message;
use std::net;
use std::thread;
use std::io::prelude::*;
use std::sync::{Mutex, Arc};
// use std::sync::mpsc;
use std::collections::HashMap;

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
            listener        : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
            server_version  : Arc::new(1),
            peers           : Arc::new(Mutex::new(HashMap::new())),
            // sender          : tx,
            // receiver        : rx,
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap().try_clone().unwrap();
            self.peers.lock().unwrap().insert(stream.peer_addr().unwrap().ip(), stream.try_clone().unwrap());
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
                    stream.read(&mut message_type).unwrap();
                    stream.read(&mut length).unwrap();

                    let size : u64 = deserialize(&length).unwrap();
                    let mut string = [12, 0, 0, 0, 0, 0, 0, 0].to_vec();
                    string.append(&mut message_type.to_vec());
                    let message_type : &str = deserialize(&string).unwrap();

                    if !i_know_you {
                        match message_type {
                            "whoami" => {
                                connection_version = message::WhoAmI::new(&stream).handle(&stream, 1).unwrap();
                            },
                            "whoamiack" => {
                                i_know_you = true;
                            },
                            _ => { break; }
                        }

                    }
                    else {
                        if size > 0 {
                            match message_type {
                                "getaddr" => {},
                                "addr" => {},
                                _ => ()
                            }
                        }
                    }
                }
                peers.lock().unwrap().remove(&stream.peer_addr().unwrap().ip());
            }).unwrap();
            dbg!(&self.peers);
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
                        let peer = TcpStream::connect(std::net::SocketAddr::new(ip.parse().unwrap(), 4224));
                        match peer {
                            Ok(peer)    => {
                                peers.lock().unwrap().insert(peer.peer_addr().unwrap().ip(), peer);

                            },
                            Err(oops)   => println!("Could not connect to {} .\n Error: {:?}", &ip, oops)
                        }
                    }
                    _ => ()
                }
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

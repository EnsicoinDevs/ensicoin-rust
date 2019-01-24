// use blockchain::blockchain::Blockchain;
// use model::block::Block;
use std::net::TcpStream;
use bincode::{serialize, deserialize};
use model::transaction::Transaction;
use model::message;
use std::net;
use std::thread;
use std::io::prelude::*;
use std::sync::Arc;

pub struct Server {
    pub listener        : net::TcpListener,
    pub server_version  : Arc<u32>,
    pub peers           : Vec<TcpStream>,
    // pub blockchain  : Blockchain,
    pub mempool         : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        Server {
            listener        : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
            server_version  : Arc::new(1),
            peers           : Vec::new(),
            // blockchain  : Blockchain::new(Block::genesis_block()),
            mempool         : Vec::new()
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap().try_clone().unwrap();
            self.peers.push(stream.try_clone().unwrap());
            let mut connection_version = Arc::clone(&self.server_version);
            thread::spawn(move || {
                //read message
                let mut magic : [u8; 4] = [0; 4];
                let mut message_type : [u8; 12] = [0; 12];
                let mut length : [u8; 8] = [0; 8];

                loop {
                    stream.read(&mut magic).unwrap();
                    stream.read(&mut message_type).unwrap();
                    stream.read(&mut length).unwrap();

                    let size : u64 = deserialize(&length).unwrap();
                    let mut string = [12, 0, 0, 0, 0, 0, 0, 0].to_vec();
                    string.append(&mut message_type.to_vec());
                    let message_type : &str = deserialize(&string).unwrap();
                    if size > 0 {
                        match message_type {
                            "whoami" => {connection_version = Arc::new(message::WhoAmI::new(&stream).handle(&stream, 1));},
                            "whoamiack" => {message::WhoAmIAck::handle(&stream)},
                            "getaddr" => {},
                            "addr" => {},
                            _ => ()
                        }
                    }
                }

            });
            dbg!(&self.peers);
        }
    }
}

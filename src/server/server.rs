// use blockchain::blockchain::Blockchain;
// use model::block::Block;
use bincode::{serialize, deserialize};
use model::transaction::Transaction;
use model::message;
use std::net;
use std::thread;
use std::io::prelude::*;

pub struct Server {
    pub listener    : net::TcpListener,
    pub peers       : Vec<net::TcpStream>,
    pub max_peers   : u64,
    // pub blockchain  : Blockchain,
    pub mempool     : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener    : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
            peers       : Vec::new(),
            max_peers   : 42,
            // blockchain  : Blockchain::new(Block::genesis_block()),
            mempool     : Vec::new()
        };
        return server;
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            thread::spawn(|| {
                //read message
                let mut stream = stream.unwrap();
                let mut magic : [u8; 4] = [0; 4];
                let mut message_type : [u8; 12] = [0; 12];
                let mut length : [u8; 8] = [0; 8];

                stream.read_exact(&mut magic).unwrap();
                stream.read(&mut message_type).unwrap();
                stream.read(&mut length).unwrap();

                let size : u64 = deserialize(&length).unwrap();
                let message_type : String = deserialize(&message_type).unwrap();
                let message_type = message_type.as_str();
                if size > 0 {
                    match message_type {
                        "whoami" => {message::Who_am_i::new(&stream);},
                        "whoamiack" => {},
                        "getaddr" => {},
                        "addr" => {},
                        _ => ()
                    }
                }
                // let mut payload : [u8; size];
            });
        }
    }
}

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
    // pub blockchain  : Blockchain,
    pub mempool     : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener    : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
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
                stream.read_exact(&mut message_type).unwrap();
                stream.read_exact(&mut length).unwrap();

                let size : u64 = deserialize(&length).unwrap();
                let mut string = [12, 0, 0, 0, 0, 0, 0, 0].to_vec();
                string.append(&mut message_type.to_vec());
                let message_type : &str = deserialize(&string).unwrap();
                if size > 0 {
                    match message_type {
                        "whoami" => {message::WhoAmI::new(&stream);},
                        "whoamiack" => {message::WhoAmIAck::handle(&stream)},
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

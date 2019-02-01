extern crate sha2;
extern crate serde;
extern crate bincode;
mod blockchain;
mod model;
mod server;
use std::error::Error;
use server::server::Server;
use std::thread;
use std::sync::mpsc::channel;
// use blockchain::blockchain::Blockchain;
// use model::block::Block;


fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new();
    server.start();
    server.listen();
    Ok(())
}

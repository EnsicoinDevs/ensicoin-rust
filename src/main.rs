extern crate sha2;
extern crate serde;
extern crate bincode;
mod blockchain;
mod model;
mod server;
use std::error::Error;
use server::server::Server;
// use blockchain::blockchain::Blockchain;
// use model::block::Block;
// use std::io::Read;
// use std::io::Write;
// use std::net;

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new();
    server.listen();
    Ok(())
}

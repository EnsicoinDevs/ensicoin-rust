extern crate sha2;
extern crate serde;
extern crate bincode;
mod blockchain;
mod model;
mod server;
use std::error::Error;
use server::server::Server;


fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new();
    server.start();
    server.listen();
    Ok(())
}

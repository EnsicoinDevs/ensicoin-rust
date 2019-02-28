#[macro_use]
extern crate clap;
extern crate sha2;
extern crate serde;
extern crate bincode;
mod utils;
mod blockchain;
mod model;
mod server;
use std::error::Error;
use server::server::Server;
use utils::clp;


fn main() -> Result<(), Box<dyn Error>> {
    let cli = clp::initiate_cli();
    let mut server = Server::new(cli.port);
    server.interactive();
    server.listen();
    Ok(())
}

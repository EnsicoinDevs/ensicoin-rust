<<<<<<< HEAD
#![feature(async_await)]
mod init;
mod server;
=======
pub mod init;
mod network;
>>>>>>> master
use std::error::Error;
use network::server::Server;
use utils::clp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // matrix::Http::new().register("hey", "secret");
    let args = clp::args();
    init::read_config()?;
    let mut server = Server::new(args.port);
    server.interactive();
    server.listen();
    Ok(())
}

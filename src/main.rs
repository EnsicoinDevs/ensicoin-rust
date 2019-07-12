pub mod init;
mod server;
use std::error::Error;
use server::server::Server;
use utils::clp;

fn main() -> Result<(), Box<dyn Error>> {
    // matrix::Http::new().register("hey", "secret");
    let args = clp::args();
    init::read_config()?;
    let mut server = Server::new(args.port);
    server.interactive();
    server.listen();
    Ok(())
}

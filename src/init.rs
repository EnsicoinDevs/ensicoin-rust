use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufWriter, BufReader};
use dirs::data_dir;

use utils::Error;
use blockchain::Blockchain;

#[derive(Deserialize, Serialize)]
struct Config {
    blockchain_exists: bool,
    matrix_access_token: String,
}

pub fn read_config() -> Result<(), Error> {
    let mut path = data_dir().unwrap();
    path.push("ensicoin-rust/");
    path.push("config.json");
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);

    let mut config : Config = serde_json::from_reader(reader).unwrap();
    if !config.blockchain_exists {
        Blockchain::add_genesis_block()?;
        config.blockchain_exists = true;

        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("config.json");
        let f = std::fs::OpenOptions::new().truncate(true).write(true).open(path).unwrap();
        let writer = BufWriter::new(f);
        serde_json::to_writer_pretty(writer, &config).unwrap();
    }
    Ok(())
}

use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufWriter, BufReader};
use dirs::data_dir;

use crate::utils::Error;
use crate::blockchain::Blockchain;

#[derive(Deserialize, Serialize)]
struct Config {
    blockchain_exists: bool,
}

pub fn read_config() -> Result<(), Error> {
    let mut path = data_dir()?;
    path.push("ensicoin-rust/");
    path.push("config.json");
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);

    let mut config : Config = serde_json::from_reader(reader).unwrap();
    if !config.blockchain_exists {
        Blockchain::add_genesis_block()?;
        config.blockchain_exists = true;

        let mut path = data_dir()?;
        path.push("ensicoin-rust/");
        path.push("config.json");
        let f = File::open(path).unwrap();
        let writer = BufWriter::new(f);
        serde_json::to_writer_pretty(writer, &config).unwrap();
    }
    Ok(())
}
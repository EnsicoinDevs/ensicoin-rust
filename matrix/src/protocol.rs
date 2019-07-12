use dirs::data_dir;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use crate::hash_map;
use crate::types::*;

#[derive(Deserialize, Serialize)]
pub struct Config {
    blockchain_exists: bool,
    matrix_access_token: String,
}

pub struct Http {
    client: Client,
    username: String,
    password: String,
    access_token: String,
}

impl Http {

    pub fn new(username: String, password: String) -> Self {
        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("config.json");
        let f = File::open(path).unwrap();
        let reader = BufReader::new(f);

        let config : Config = serde_json::from_reader(reader).unwrap();

        Self {
            client: Client::new(),
            username,
            password,
            access_token: config.matrix_access_token,
        }
    }

    pub fn register(&self) -> Result<(), Error> {
        let body = hash_map!(
                        "auth" => "{\"type\": \"m.login.dummy\"}",
                        "password" => &self.password,
                        "username" => &self.username
                    );


        let mut resp = self.client.post("https://matrix.org/_matrix/client/r0")
        .json(&body)
        .send()?;

        match resp.status() {
            StatusCode::OK => {
                let data: RegisterResponse = resp.json()?;
                // store access_token into creditentials file
                let mut path = data_dir().unwrap();
                path.push("ensicoin-rust/");
                path.push("config.json");
                let f = File::open(path).unwrap();
                let reader = BufReader::new(f);

                let mut config : Config = serde_json::from_reader(reader).unwrap();
                config.matrix_access_token = data.access_token().to_string();

                let mut path = data_dir().unwrap();
                path.push("ensicoin-rust/");
                path.push("config.json");
                let f = File::open(path).unwrap();
                let writer = BufWriter::new(f);
                serde_json::to_writer_pretty(writer, &config).unwrap();
            },
            _ => return Err(Error::StatusCodeError),
        }

        Ok(())
    }

}

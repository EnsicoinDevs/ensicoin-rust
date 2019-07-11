use reqwest::Client;
use reqwest::StatusCode;
use crate::hash_map;
use crate::types::*;

pub struct Http {
    client: Client,
    username: String,
    password: String,
    access_token: String,
}

impl Http {

    pub fn new(username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            username,
            password,
            access_token: "".to_owned(),
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
                let _data: RegisterResponse = resp.json()?;
                // store access_token into creditentials file
            },
            _ => return Err(Error::StatusCodeError),
        }

        Ok(())
    }

    pub fn login(&self) -> Result<(), Error> {
        let mut resp = self.client.get("https://matrix.org/_matrix/client/r0/login")
        .send()?;

        match resp.status() {
            StatusCode::OK => {
                ()
            },
            _ => return Err(Error::StatusCodeError),
        }
        Ok(())
    }
}

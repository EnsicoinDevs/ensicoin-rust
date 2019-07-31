use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RegisterResponse {
    access_token: String,
    home_server: String,
    user_id: String,
}

impl RegisterResponse {
    pub fn access_token(&self) -> &String {
        &self.access_token
    }
}

pub enum Error {
    ReqwestError(reqwest::Error),
    StatusCodeError,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RegisterResponse {
    access_token: String,
    home_server: String,
    user_id: String,
}

// pub struct LoginTypeResponse {
//     flow: Vec[]
// }


pub enum Error {
    ReqwestError(reqwest::Error),
    StatusCodeError,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

#[derive(Debug)]
pub enum Error {
    DeserializeError(std::boxed::Box<bincode::ErrorKind>),
    IOError(std::io::Error),
    ParseError(String),
    ConnectionClosed,
}

impl From<std::boxed::Box<bincode::ErrorKind>> for Error {
    fn from(e : std::boxed::Box<bincode::ErrorKind>) -> Error {
        Error::DeserializeError(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e : std::io::Error) -> Error {
        Error::IOError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_e : std::string::FromUtf8Error) -> Error {
        Error::ParseError("String from byte array failed".to_string())
    }
}

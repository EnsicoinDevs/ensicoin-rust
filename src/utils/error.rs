#[derive(Debug)]
pub enum Error {
    DeserializeError(std::boxed::Box<bincode::ErrorKind>),
    IOError(std::io::Error),
    ParseError(String),
    DBError,
    ConnectionClosed,
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error encountered")
    }
}

impl std::error::Error for Error {
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
    fn from(_ : std::string::FromUtf8Error) -> Error {
        Error::ParseError("String from byte array failed".to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Error {
        Error::ConnectionClosed
    }
}

impl From<sled::Error> for Error {
    fn from(_: sled::Error) -> Error {
        Error::DBError
    }
}

impl From<std::option::NoneError> for Error {
    fn from(_: std::option::NoneError) -> Error {
        Error::DBError
    }
}

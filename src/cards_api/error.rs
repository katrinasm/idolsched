use std::fmt::{self, Formatter, Debug, Display};

use Error::*;

#[derive(Debug)]
pub enum Error {
    ParseError(serde_json::Error),
    RequestError(reqwest::Error),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError(e) => write!(f, "couldn't parse json: {}", e),
            RequestError(e) => write!(f, "network request error: {}", e),
            IoError(e) => write!(f, "error working with cache: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError(e) => Some(e),
            RequestError(e) => Some(e),
            IoError(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        ParseError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        RequestError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        IoError(e)
    }
}


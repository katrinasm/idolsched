use std::fmt::{self, Formatter, Debug, Display};
use super::cards_api;

use Error::*;

#[derive(Debug)]
pub enum Error {
    ParseError(serde_json::Error),
    ApiError(cards_api::Error),
    BadCmdLine(getopts::Fail),
    IoError(std::io::Error),
    StrErr(&'static str),
    Etc(Box<dyn std::error::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError(e) => write!(f, "couldn't parse json: {}", e),
            ApiError(e) => write!(f, "cards API: {}", e),
            BadCmdLine(e) => write!(f, "incorrect command line argument: {}", e),
            IoError(e) => write!(f, "i/o: {}", e),
            StrErr(s) => write!(f, "{}", s),
            Etc(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError(e) => Some(e),
            ApiError(e) => Some(e),
            BadCmdLine(e) => Some(e),
            IoError(e) => Some(e),
            StrErr(_) => None,
            Etc(_) => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        ParseError(e)
    }
}

impl From<cards_api::Error> for Error {
    fn from(e: cards_api::Error) -> Error {
        ApiError(e)
    }
}

impl From<getopts::Fail> for Error {
    fn from(e: getopts::Fail) -> Error {
        BadCmdLine(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        IoError(e)
    }
}


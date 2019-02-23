use std::{io, result, fmt};

use xml::{reader, writer};
use curl;


#[derive(Debug)]
pub enum Error {
    Custom(String),
    Reader(reader::Error),
    Writer(writer::Error),
    Io(io::Error),
    Curl(curl::Error)
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Custom(ref e) => write!(f, "{}", e),
            Error::Reader(ref e) => reader::Error::fmt(e, f),
            Error::Writer(ref e) => writer::Error::fmt(e, f),
            Error::Io(ref e) => io::Error::fmt(e, f),
            Error::Curl(ref e) => curl::Error::fmt(e, f),
        }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<reader::Error> for Error {
    fn from(e: reader::Error) -> Self {
        Error::Reader(e)
    }
}

impl From<writer::Error> for Error {
    fn from(e: writer::Error) -> Self {
        Error::Writer(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<curl::Error> for Error {
    fn from(e: curl::Error) -> Self {
        Error::Curl(e)
    }
}

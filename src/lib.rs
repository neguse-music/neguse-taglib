#[macro_use] extern crate lazy_static;

mod types;
use crate::types::DateTime;
use crate::types::Image;
use crate::types::TagOption;
use crate::types::Tags;

mod flac;
mod id3v1;
mod id3v2;
mod m4a;

mod vorbis;

mod dispatch;
mod tools;

#[cfg(test)]
mod tests;

pub use crate::dispatch::get_front_cover;
pub use crate::dispatch::get_tags;
pub use crate::dispatch::set_tags;

use std::io;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    TagError(String),
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref e) => write!(f, "IO error: {}", e),
            Error::TagError(ref e) => write!(f, "Error reading tag from file: {}", e),
        }
    }
}

use std::error;
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IOError(ref e) => e.description(),
            Error::TagError(ref e) => e.as_str(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::IOError(ref e) => Some(e),
            Error::TagError(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}
impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::TagError(err)
    }
}

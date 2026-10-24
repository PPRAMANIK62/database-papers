//! `dbparse` reads the SQLite database file format, following the spec at
//! <https://www.sqlite.org/fileformat2.html>. It only reads, never writes.

use std::{error, fmt, io, result};

mod bytes;
mod pager;

/// Everything that can go wrong while reading a database file.
#[derive(Debug)]
pub enum Error {
    /// The operating system refused a read.
    Io(io::Error),
    /// The bytes break a rule of the file format.
    Corrupt(String),
    /// A table or index name that isn't in the schema.
    NotFound(String),
}

/// What every read returns: the value, or the [`Error`] that stopped it.
pub type Result<T> = result::Result<T, Error>;

/// A shorthand for returning `Error::Corrupt`.
pub(crate) fn corrupt<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Corrupt(message.into()))
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Corrupt(m) => write!(f, "corrupt database: {m}"),
            Error::NotFound(name) => write!(f, "no table or index named {name:?}"),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

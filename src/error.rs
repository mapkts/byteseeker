use std::{error, fmt, io, result};

/// A type alias for `Result<T, byteSeeker::Error>`.
///
/// This result type embeds the error type in this crate.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when seeking bytes.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// A crate private constructor for `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    /// Returns the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwraps this error into its undelying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// The specific type of an error.
///
/// This list might grow over time and it is not recommended to
/// exhaustively match against it.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Represents an I/O error.
    ///
    /// Can occur when reading the underlying byte stream.
    Io(io::Error),
    /// The seeking byte slice is not found within the given byte stream.
    ByteNotFound,
    /// The length of the given byte slice is zero,
    /// or excesses the capacity of `ByteSeeker`.
    UnsupportedLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::ByteNotFound => write!(f, "Byte not found"),
            ErrorKind::UnsupportedLength => write!(
                f,
                "The length of the given byte slice is zero, or excesses the capacity of `ByteSeeker`"
            ),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(ErrorKind::Io(err))
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

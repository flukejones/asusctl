use std::fmt;

use rog_anime::error::AnimeError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ConfigLoadFail,
    ConfigLockFail,
    XdgVars,
    Anime(AnimeError),
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "Failed to open: {}", err),
            Error::ConfigLoadFail => write!(f, "Failed to load user config"),
            Error::ConfigLockFail => write!(f, "Failed to lock user config"),
            Error::XdgVars => write!(f, "XDG environment vars appear unset"),
            Error::Anime(err) => write!(f, "Anime error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<AnimeError> for Error {
    fn from(err: AnimeError) -> Self {
        Error::Anime(err)
    }
}

impl From<Error> for zbus::fdo::Error {
    fn from(err: Error) -> Self {
        zbus::fdo::Error::Failed(format!("Anime zbus error: {}", err))
    }
}

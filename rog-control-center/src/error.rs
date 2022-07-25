use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Nix(nix::Error),
    ConfigLoadFail,
    ConfigLockFail,
    XdgVars,
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "Failed to open: {}", err),
            Error::Nix(err) => write!(f, "Error: {}", err),
            Error::ConfigLoadFail => write!(f, "Failed to load user config"),
            Error::ConfigLockFail => write!(f, "Failed to lock user config"),
            Error::XdgVars => write!(f, "XDG environment vars appear unset"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<Error> for zbus::fdo::Error {
    fn from(err: Error) -> Self {
        zbus::fdo::Error::Failed(format!("Anime zbus error: {}", err))
    }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Self {
        Error::Nix(err)
    }
}

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Nix(nix::Error),
    ConfigLoadFail,
    ConfigLockFail,
    XdgVars,
    Zbus(zbus::Error),
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
            Error::Zbus(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Self {
        Error::Nix(err)
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::Zbus(err)
    }
}

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ConfigLoadFail,
    ConfigLockFail,
    XdgVars,
    Zbus(zbus::Error),
    ZbusFdo(zbus::fdo::Error),
    Notification(notify_rust::error::Error),
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "Failed to open: {}", err),
            Error::ConfigLoadFail => write!(f, "Failed to load user config"),
            Error::ConfigLockFail => write!(f, "Failed to lock user config"),
            Error::XdgVars => write!(f, "XDG environment vars appear unset"),
            Error::Zbus(err) => write!(f, "Error: {}", err),
            Error::ZbusFdo(err) => write!(f, "Error: {}", err),
            Error::Notification(err) => write!(f, "Notification Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::Zbus(err)
    }
}

impl From<zbus::fdo::Error> for Error {
    fn from(err: zbus::fdo::Error) -> Self {
        Error::ZbusFdo(err)
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(err: notify_rust::error::Error) -> Self {
        Error::Notification(err)
    }
}

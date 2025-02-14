use std::error::Error;
use std::fmt;
pub type Result<T> = std::result::Result<T, SlashError>;

#[derive(Debug)]
pub enum SlashError {
    Dbus(String),
    Udev(String, std::io::Error),
    NoDevice,
    UnsupportedDevice,
    DataBufferLength,
    ParseError(String),
}

impl fmt::Display for SlashError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlashError::ParseError(e) => write!(f, "Could not parse {e}"),
            SlashError::Dbus(detail) => write!(f, "{}", detail),
            SlashError::Udev(deets, error) => write!(f, "udev {}: {}", deets, error),
            SlashError::NoDevice => write!(f, "No Slash device found"),
            SlashError::DataBufferLength => write!(
                f,
                "The data buffer was incorrect length for generating USB packets"
            ),
            SlashError::UnsupportedDevice => write!(f, "Unsupported Slash device found"),
        }
    }
}

impl Error for SlashError {}

impl From<SlashError> for zbus::fdo::Error {
    #[inline]
    fn from(err: SlashError) -> Self {
        zbus::fdo::Error::Failed(format!("{}", err))
    }
}

use std::fmt;

pub type Result<T> = std::result::Result<T, PlatformError>;

#[derive(Debug)]
pub enum PlatformError {
    ParseVendor,
    ParseNum,
    Udev(String, std::io::Error),
    USB(rusb::Error),
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    AttrNotFound(String),
    MissingFunction(String),
    MissingLedBrightNode(String, std::io::Error),
    IoPath(String, std::io::Error),
    Io(std::io::Error),
    NoAuraKeyboard,
    NoAuraNode,
}

impl fmt::Display for PlatformError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlatformError::ParseVendor => write!(f, "Parse gfx vendor error"),
            PlatformError::ParseNum => write!(f, "Parse number error"),
            PlatformError::Udev(deets, error) => write!(f, "udev {}: {}", deets, error),
            PlatformError::USB(error) => write!(f, "usb {}", error),
            PlatformError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            PlatformError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            PlatformError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            PlatformError::NotSupported => write!(f, "Not supported"),
            PlatformError::AttrNotFound(deets) => write!(f, "Attribute not found: {}", deets),
            PlatformError::Io(deets) => write!(f, "std::io error: {}", deets),
            PlatformError::MissingFunction(deets) => write!(f, "Missing functionality: {}", deets),
            PlatformError::MissingLedBrightNode(path, error) => write!(f, "Led node at {} is missing, please check you have the required patch or dkms module installed: {}", path, error),
            PlatformError::IoPath(path, detail) => write!(f, "{} {}", path, detail),
            PlatformError::NoAuraKeyboard => write!(f, "No supported Aura keyboard"),
            PlatformError::NoAuraNode => write!(f, "No Aura keyboard node found"),
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<rusb::Error> for PlatformError {
    fn from(err: rusb::Error) -> Self {
        PlatformError::USB(err)
    }
}

impl From<std::io::Error> for PlatformError {
    fn from(err: std::io::Error) -> Self {
        PlatformError::Io(err)
    }
}

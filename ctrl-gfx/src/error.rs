use std::error;
use std::fmt;

#[derive(Debug)]
pub enum GfxError {
    ParseVendor,
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    Module(String, std::io::Error),
    Bus(String, std::io::Error),
    Command(String, std::io::Error),
}

impl fmt::Display for GfxError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GfxError::ParseVendor => write!(f, "Could not parse vendor name"),
            GfxError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            GfxError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            GfxError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            GfxError::Module(func, error) => write!(f, "Module error: {}: {}", func, error),
            GfxError::Bus(func, error) => write!(f, "Bus error: {}: {}", func, error),
            GfxError::Command(func, error) => write!(f, "Command exec error: {}: {}", func, error),
        }
    }
}

impl error::Error for GfxError {}

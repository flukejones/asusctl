use std::error;
use std::fmt;

use crate::error::RogError;

#[derive(Debug)]
pub enum GfxError {
    ParseVendor,
    Bus(String, std::io::Error),
    DisplayManager(String),
}

impl fmt::Display for GfxError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GfxError::ParseVendor => write!(f, "Could not parse vendor name"),
            GfxError::Bus(func, error) => write!(f, "Bus error: {}: {}", func, error),
            GfxError::DisplayManager(detail) => write!(f, "Display manager: {}", detail),
        }
    }
}

impl error::Error for GfxError {}

impl From<GfxError> for RogError {
    fn from(err: GfxError) -> Self {
        RogError::GfxSwitching(err)
    }
}

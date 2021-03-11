use std::fmt;
use std::{error, process::ExitStatus};

use crate::error::RogError;

#[derive(Debug)]
pub enum GfxError {
    ParseVendor,
    Bus(String, std::io::Error),
    DisplayManagerAction(String, ExitStatus),
    DisplayManagerTimeout(String),
    GsyncModeActive,
}

impl fmt::Display for GfxError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GfxError::ParseVendor => write!(f, "Could not parse vendor name"),
            GfxError::Bus(func, error) => write!(f, "Bus error: {}: {}", func, error),
            GfxError::DisplayManagerAction(action, status) => {
                write!(f, "Display-manager action {} failed: {}", action, status)
            }
            GfxError::DisplayManagerTimeout(state) => {
                write!(f, "Timed out waiting for display-manager {} state", state)
            }
            GfxError::GsyncModeActive => write!(
                f,
                "Can not switch gfx modes when dedicated/G-Sync mode is active"
            ),
        }
    }
}

impl error::Error for GfxError {}

impl From<GfxError> for RogError {
    fn from(err: GfxError) -> Self {
        RogError::GfxSwitching(err)
    }
}

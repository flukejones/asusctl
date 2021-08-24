use std::fmt;
use std::{error, process::ExitStatus};

#[derive(Debug)]
pub enum GfxError {
    ParseVendor,
    ParsePower,
    Bus(String, std::io::Error),
    DisplayManagerAction(String, ExitStatus),
    DisplayManagerTimeout(String),
    GsyncModeActive,
    VfioBuiltin,
    VfioDisabled,
    MissingModule(String),
    Modprobe(String),
    Command(String, std::io::Error),
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    Io(std::io::Error),
    Zbus(zbus::Error),
}

impl fmt::Display for GfxError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GfxError::ParseVendor => write!(f, "Could not parse vendor name"),
            GfxError::ParsePower => write!(f, "Could not parse dGPU power status"),
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
            GfxError::VfioBuiltin => write!(
                f,
                "Can not switch to vfio mode if the modules are built in to kernel"
            ),
            GfxError::VfioDisabled => {
                write!(f, "Can not switch to vfio mode if disabled in config file")
            }
            GfxError::MissingModule(m) => write!(f, "The module {} is missing", m),
            GfxError::Modprobe(detail) => write!(f, "Modprobe error: {}", detail),
            GfxError::Command(func, error) => write!(f, "Command exec error: {}: {}", func, error),
            GfxError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            GfxError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            GfxError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            GfxError::Io(detail) => write!(f, "std::io error: {}", detail),
            GfxError::Zbus(detail) => write!(f, "Zbus error: {}", detail),
        }
    }
}

impl error::Error for GfxError {}

impl From<zbus::Error> for GfxError {
    fn from(err: zbus::Error) -> Self {
        GfxError::Zbus(err)
    }
}

impl From<std::io::Error> for GfxError {
    fn from(err: std::io::Error) -> Self {
        GfxError::Io(err)
    }
}
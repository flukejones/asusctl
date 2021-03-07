use intel_pstate::PStateError;
use rog_fan_curve::CurveError;
use rog_types::error::GraphicsError;
use std::convert::From;
use std::fmt;

use crate::ctrl_gfx::error::GfxError;

#[derive(Debug)]
pub enum RogError {
    ParseFanLevel,
    ParseVendor,
    ParseLED,
    MissingProfile(String),
    Udev(String, std::io::Error),
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    IntelPstate(PStateError),
    FanCurve(CurveError),
    DoTask(String),
    MissingFunction(String),
    MissingLedBrightNode(String, std::io::Error),
    ReloadFail(String),
    GfxSwitching(GfxError),
    Initramfs(String),
}

impl fmt::Display for RogError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RogError::ParseFanLevel => write!(f, "Parse profile error"),
            RogError::ParseVendor => write!(f, "Parse gfx vendor error"),
            RogError::ParseLED => write!(f, "Parse LED error"),
            RogError::MissingProfile(profile) => write!(f, "Profile does not exist {}", profile),
            RogError::Udev(deets, error) => write!(f, "udev {}: {}", deets, error),
            RogError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            RogError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            RogError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            RogError::NotSupported => write!(f, "Not supported"),
            RogError::NotFound(deets) => write!(f, "Not found: {}", deets),
            RogError::IntelPstate(err) => write!(f, "Intel pstate error: {}", err),
            RogError::FanCurve(err) => write!(f, "Custom fan-curve error: {}", err),
            RogError::DoTask(deets) => write!(f, "Task error: {}", deets),
            RogError::MissingFunction(deets) => write!(f, "Missing functionality: {}", deets),
            RogError::MissingLedBrightNode(path, error) => write!(f, "Led node at {} is missing, please check you have the required patch or dkms module installed: {}", path, error),
            RogError::ReloadFail(deets) => write!(f, "Task error: {}", deets),
            RogError::GfxSwitching(deets) => write!(f, "Graphics switching error: {}", deets),
            RogError::Initramfs(detail) => write!(f, "Initiramfs error: {}", detail),
        }
    }
}

impl std::error::Error for RogError {}

impl From<PStateError> for RogError {
    fn from(err: PStateError) -> Self {
        RogError::IntelPstate(err)
    }
}

impl From<CurveError> for RogError {
    fn from(err: CurveError) -> Self {
        RogError::FanCurve(err)
    }
}

impl From<GraphicsError> for RogError {
    fn from(err: GraphicsError) -> Self {
        match err {
            GraphicsError::ParseVendor => RogError::GfxSwitching(GfxError::ParseVendor),
        }
    }
}

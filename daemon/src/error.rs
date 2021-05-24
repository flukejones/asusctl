use rog_fan_curve::CurveError;
use rog_profiles::error::ProfileError;
use rog_types::error::GraphicsError;
use std::convert::From;
use std::fmt;

use crate::ctrl_gfx::error::GfxError;

#[derive(Debug)]
pub enum RogError {
    ParseFanLevel,
    ParseVendor,
    ParseLed,
    MissingProfile(String),
    Udev(String, std::io::Error),
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    FanCurve(CurveError),
    DoTask(String),
    MissingFunction(String),
    MissingLedBrightNode(String, std::io::Error),
    ReloadFail(String),
    GfxSwitching(GfxError),
    Profiles(ProfileError),
    Initramfs(String),
    Modprobe(String),
    Command(String, std::io::Error),
    Io(std::io::Error),
    Zbus(zbus::Error),
}

impl fmt::Display for RogError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RogError::ParseFanLevel => write!(f, "Parse profile error"),
            RogError::ParseVendor => write!(f, "Parse gfx vendor error"),
            RogError::ParseLed => write!(f, "Parse LED error"),
            RogError::MissingProfile(profile) => write!(f, "Profile does not exist {}", profile),
            RogError::Udev(deets, error) => write!(f, "udev {}: {}", deets, error),
            RogError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            RogError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            RogError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            RogError::NotSupported => write!(f, "Not supported"),
            RogError::NotFound(deets) => write!(f, "Not found: {}", deets),
            RogError::FanCurve(err) => write!(f, "Custom fan-curve error: {}", err),
            RogError::DoTask(deets) => write!(f, "Task error: {}", deets),
            RogError::MissingFunction(deets) => write!(f, "Missing functionality: {}", deets),
            RogError::MissingLedBrightNode(path, error) => write!(f, "Led node at {} is missing, please check you have the required patch or dkms module installed: {}", path, error),
            RogError::ReloadFail(deets) => write!(f, "Task error: {}", deets),
            RogError::GfxSwitching(deets) => write!(f, "Graphics switching error: {}", deets),
            RogError::Profiles(deets) => write!(f, "Profile error: {}", deets),
            RogError::Initramfs(detail) => write!(f, "Initiramfs error: {}", detail),
            RogError::Modprobe(detail) => write!(f, "Modprobe error: {}", detail),
            RogError::Command(func, error) => write!(f, "Command exec error: {}: {}", func, error),
            RogError::Io(detail) => write!(f, "std::io error: {}", detail),
            RogError::Zbus(detail) => write!(f, "Zbus error: {}", detail),
        }
    }
}

impl std::error::Error for RogError {}

impl From<CurveError> for RogError {
    fn from(err: CurveError) -> Self {
        RogError::FanCurve(err)
    }
}

impl From<GraphicsError> for RogError {
    fn from(err: GraphicsError) -> Self {
        match err {
            GraphicsError::ParseVendor => RogError::GfxSwitching(GfxError::ParseVendor),
            GraphicsError::ParsePower => RogError::GfxSwitching(GfxError::ParsePower),
        }
    }
}

impl From<ProfileError> for RogError {
    fn from(err: ProfileError) -> Self {
        RogError::Profiles(err)
    }
}

impl From<zbus::Error> for RogError {
    fn from(err: zbus::Error) -> Self {
        RogError::Zbus(err)
    }
}

impl From<std::io::Error> for RogError {
    fn from(err: std::io::Error) -> Self {
        RogError::Io(err)
    }
}

impl From<RogError> for zbus::fdo::Error {
    #[inline]
    fn from(err: RogError) -> Self {
        zbus::fdo::Error::Failed(format!("{}", err))
    }
}

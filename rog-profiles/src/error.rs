use std::fmt;

use intel_pstate::PStateError;
use rog_fan_curve::CurveError;

#[derive(Debug)]
pub enum ProfileError {
    ParseFanLevel,
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    IntelPstate(PStateError),
    FanCurve(CurveError),
    Io(std::io::Error),
    //Zbus(zbus::Error),
}

impl fmt::Display for ProfileError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProfileError::ParseFanLevel => write!(f, "Parse profile error"),
            ProfileError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            ProfileError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            ProfileError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            ProfileError::NotSupported => write!(f, "Not supported"),
            ProfileError::NotFound(deets) => write!(f, "Not found: {}", deets),
            ProfileError::IntelPstate(err) => write!(f, "Intel pstate error: {}", err),
            ProfileError::FanCurve(err) => write!(f, "Custom fan-curve error: {}", err),
            ProfileError::Io(detail) => write!(f, "std::io error: {}", detail),
            //Error::Zbus(detail) => write!(f, "Zbus error: {}", detail),
        }
    }
}

impl std::error::Error for ProfileError {}

impl From<PStateError> for ProfileError {
    fn from(err: PStateError) -> Self {
        ProfileError::IntelPstate(err)
    }
}

impl From<CurveError> for ProfileError {
    fn from(err: CurveError) -> Self {
        ProfileError::FanCurve(err)
    }
}

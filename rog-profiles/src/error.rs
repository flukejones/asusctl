use std::fmt;

#[derive(Debug)]
pub enum ProfileError {
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    Io(std::io::Error),
    ParseProfileName,
    ParseFanCurveDigit(std::num::ParseIntError),
    /// (pwm/temp, prev, next)
    ParseFanCurvePrevHigher(&'static str, u8, u8),
    ParseFanCurvePercentOver100(u8),
    // Zbus(zbus::Error),
}

impl fmt::Display for ProfileError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProfileError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            ProfileError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            ProfileError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            ProfileError::NotSupported => write!(f, "Not supported"),
            ProfileError::NotFound(deets) => write!(f, "Not found: {}", deets),
            ProfileError::Io(detail) => write!(f, "std::io error: {}", detail),
            ProfileError::ParseProfileName => write!(f, "Invalid profile name"),
            ProfileError::ParseFanCurveDigit(e) => {
                write!(f, "Could not parse number to 0-255: {}", e)
            }
            ProfileError::ParseFanCurvePrevHigher(part, prev, next) => write!(
                f,
                "Invalid {}, previous value {} is higher than next value {}",
                part, prev, next
            ),
            ProfileError::ParseFanCurvePercentOver100(value) => {
                write!(f, "Invalid percentage, {} is higher than 100", value)
            }
            // Error::Zbus(detail) => write!(f, "Zbus error: {}", detail),
        }
    }
}

impl std::error::Error for ProfileError {}

impl From<std::io::Error> for ProfileError {
    fn from(err: std::io::Error) -> Self {
        ProfileError::Io(err)
    }
}

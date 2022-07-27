use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParseColour,
    ParseSpeed,
    ParseDirection,
    ParseBrightness,
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseColour => write!(f, "Could not parse colour"),
            Error::ParseSpeed => write!(f, "Could not parse speed"),
            Error::ParseDirection => write!(f, "Could not parse direction"),
            Error::ParseBrightness => write!(f, "Could not parse brightness"),
            Error::Io(io) => write!(f, "IO Error: {io}"),
            Error::Toml(e) => write!(f, "TOML Parse Error: {e}"),
        }
    }
}

impl error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParseColour,
    ParseSpeed,
    ParseDirection,
    ParseBrightness,
    ParseAnime,
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseColour => write!(f, "Could not parse colour"),
            Error::ParseSpeed => write!(f, "Could not parse speed"),
            Error::ParseDirection => write!(f, "Could not parse direction"),
            Error::ParseBrightness => write!(f, "Could not parse brightness"),
            Error::ParseAnime => write!(f, "Could not parse anime"),
        }
    }
}

impl error::Error for Error {}

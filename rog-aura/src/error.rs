use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    ParseColour,
    ParseSpeed,
    ParseDirection,
    ParseBrightness,
    IoPath(String, std::io::Error),
    Ron(ron::Error),
    RonParse(ron::error::SpannedError),
}

impl fmt::Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseColour => write!(f, "Could not parse colour"),
            Error::ParseSpeed => write!(f, "Could not parse speed"),
            Error::ParseDirection => write!(f, "Could not parse direction"),
            Error::ParseBrightness => write!(f, "Could not parse brightness"),
            Error::IoPath(path, io) => write!(f, "IO Error: {path}, {io}"),
            Error::Ron(e) => write!(f, "RON Parse Error: {e}"),
            Error::RonParse(e) => write!(f, "RON Parse Error: {e}"),
        }
    }
}

impl error::Error for Error {}

impl From<ron::Error> for Error {
    fn from(e: ron::Error) -> Self {
        Self::Ron(e)
    }
}

impl From<ron::error::SpannedError> for Error {
    fn from(e: ron::error::SpannedError) -> Self {
        Self::RonParse(e)
    }
}

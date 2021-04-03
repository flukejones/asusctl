use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AuraError {
    ParseColour,
    ParseSpeed,
    ParseDirection,
    ParseBrightness,
    ParseAnime,
}

impl fmt::Display for AuraError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuraError::ParseColour => write!(f, "Could not parse colour"),
            AuraError::ParseSpeed => write!(f, "Could not parse speed"),
            AuraError::ParseDirection => write!(f, "Could not parse direction"),
            AuraError::ParseBrightness => write!(f, "Could not parse brightness"),
            AuraError::ParseAnime => write!(f, "Could not parse anime"),
        }
    }
}

impl Error for AuraError {}

#[derive(Debug)]
pub enum GraphicsError {
    ParseVendor,
    ParsePower,
}

impl fmt::Display for GraphicsError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphicsError::ParseVendor => write!(f, "Could not parse vendor name"),
            GraphicsError::ParsePower => write!(f, "Could not parse dGPU power status"),
        }
    }
}

impl Error for GraphicsError {}

#[derive(Debug)]
pub enum AnimeError {
    InvalidBitmap,
    Io(std::io::Error),
}

impl fmt::Display for AnimeError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnimeError::InvalidBitmap => write!(f, "Bitmap is invalid"),
            AnimeError::Io(e) => write!(f, "Could not open: {}", e),
        }
    }
}

impl Error for AnimeError {}

impl From<std::io::Error> for AnimeError {
    fn from(err: std::io::Error) -> Self {
        AnimeError::Io(err)
    }
}

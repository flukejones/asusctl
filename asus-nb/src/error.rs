use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AuraError {
    ParseColour,
    ParseSpeed,
    ParseDirection,
    ParseBrightness,
}

impl fmt::Display for AuraError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuraError::ParseColour => write!(f, "Could not parse colour"),
            AuraError::ParseSpeed => write!(f, "Could not parse speed"),
            AuraError::ParseDirection => write!(f, "Could not parse direction"),
            AuraError::ParseBrightness => write!(f, "Could not parse brightness"),
        }
    }
}

impl Error for AuraError {}

#[derive(Debug)]
pub enum GraphicsError {
    ParseVendor,
}

impl fmt::Display for GraphicsError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphicsError::ParseVendor => write!(f, "Could not parse vendor name"),
        }
    }
}

impl Error for GraphicsError {}

use std::fmt;

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

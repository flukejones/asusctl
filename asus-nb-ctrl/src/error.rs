use std::fmt;

#[derive(Debug)]
pub enum RogError {
    ParseFanLevel,
    MissingProfile(String),
    NotSupported,
}

impl std::error::Error for RogError {}

impl fmt::Display for RogError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RogError::ParseFanLevel => write!(f, "Parse error"),
            RogError::MissingProfile(profile) => write!(f, "Profile does not exist {}", profile),
            RogError::NotSupported => write!(f, "Not supported"),
        }
    }
}

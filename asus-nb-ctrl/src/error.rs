use std::fmt;

#[derive(Debug)]
pub enum RogError {
    ParseFanLevel,
    NotSupported,
}

impl std::error::Error for RogError {}

impl fmt::Display for RogError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RogError::ParseFanLevel => write!(f, "Parse error"),
            RogError::NotSupported => write!(f, "Not supported"),
        }
    }
}

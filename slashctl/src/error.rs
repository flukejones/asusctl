use std::fmt;
use std::fmt::{Display};
use rog_slash::error::SlashError;

#[derive(Debug)]
pub enum SlashCtlError {
    NotSupported,
    Slash(SlashError),
}


impl Display for SlashCtlError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlashCtlError::NotSupported => write!(f, "Not supported"),
            SlashCtlError::Slash(err) => write!(f, "Slash error: {}", err),
        }
    }
}

impl std::error::Error for SlashCtlError {}



impl From<SlashError> for SlashCtlError {
    fn from(err: SlashError) -> Self {
        SlashCtlError::Slash(err)
    }
}
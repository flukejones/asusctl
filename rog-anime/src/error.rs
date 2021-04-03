use std::error::Error;
use std::fmt;
use png_pong::decode::Error as PngError;

#[derive(Debug)]
pub enum AnimeError {
    NoFrames,
    Io(std::io::Error),
    Png(PngError),
    Format
}

impl fmt::Display for AnimeError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnimeError::NoFrames => write!(f, "No frames in PNG"),
            AnimeError::Io(e) => write!(f, "Could not open: {}", e),
            AnimeError::Png(e) => write!(f, "PNG error: {}", e),
            AnimeError::Format => write!(f, "PNG file is not 8bit greyscale"),
        }
    }
}

impl Error for AnimeError {}

impl From<std::io::Error> for AnimeError {
    fn from(err: std::io::Error) -> Self {
        AnimeError::Io(err)
    }
}

impl From<PngError> for AnimeError {
    fn from(err: PngError) -> Self {
        AnimeError::Png(err)
    }
}
use std::fmt;

#[derive(Debug)]
pub enum ProfileError {
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    Io(std::io::Error),
    //Zbus(zbus::Error),
}

impl fmt::Display for ProfileError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProfileError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            ProfileError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            ProfileError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            ProfileError::NotSupported => write!(f, "Not supported"),
            ProfileError::NotFound(deets) => write!(f, "Not found: {}", deets),
            ProfileError::Io(detail) => write!(f, "std::io error: {}", detail),
            //Error::Zbus(detail) => write!(f, "Zbus error: {}", detail),
        }
    }
}

impl std::error::Error for ProfileError {}

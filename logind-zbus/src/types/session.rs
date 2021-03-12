#[derive(Debug, PartialEq)]
pub enum SessionType {
    X11,
    Wayland,
    TTY,
    Other(String),
}

impl From<String> for SessionType {
    fn from(s: String) -> Self {
        <SessionType>::from(s.as_str())
    }
}

impl From<&str> for SessionType {
    fn from(s: &str) -> Self {
        match s {
            "wayland" => SessionType::Wayland,
            "x11" =>  SessionType::X11,
            "tty" => SessionType::TTY,
            _ => SessionType::Other(s.to_owned()),
        }
    }
}
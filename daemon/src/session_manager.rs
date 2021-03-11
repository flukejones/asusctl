use log::{error, warn};
use serde_derive::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, PartialEq)]
enum SessionType {
    X11,
    Wayland,
    TTY,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserSession {
    pub session: String,
    pub uid: u32,
    pub user: String,
    pub seat: String,
    pub tty: String,
}

#[derive(Debug)]
pub struct Session {
    session_id: String,
    session_type: SessionType,
}

pub fn are_gfx_sessions_alive(sessions: &[Session]) -> bool {
    for session in sessions {
        match is_gfx_alive(session) {
            Ok(alive) => {
                if alive {
                    return true;
                }
            }
            Err(err) => warn!("Error checking sessions: {}", err),
        }
    }
    false
}

pub fn get_sessions() -> Result<Vec<Session>, SessionError> {
    // loginctl list-sessions --no-legend
    let mut cmd = Command::new("loginctl");
    cmd.arg("list-sessions");
    cmd.arg("--output");
    cmd.arg("json");

    let mut sessions = Vec::new();

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                error!(
                    "Couldn't get sessions: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else if output.status.success() {
                if let Ok(data) = serde_json::from_slice::<Vec<UserSession>>(&output.stdout) {
                    for s in &data {
                        if let Ok(t) = get_session_type(&s.session) {
                            sessions.push(Session {
                                session_id: s.session.to_owned(),
                                session_type: t,
                            })
                        }
                    }
                    return Ok(sessions);
                }
            }
        }
        Err(err) => error!("Couldn't get sessions: {}", err),
    }
    Err(SessionError::NoSessions)
}

fn is_gfx_alive(session: &Session) -> Result<bool, SessionError> {
    if session.session_type == SessionType::TTY {
        return Ok(false);
    }
    let session_id = session.session_id.to_owned();
    let mut cmd = Command::new("loginctl");
    cmd.arg("show-session");
    cmd.arg(&session_id);
    cmd.arg("--property");
    cmd.arg("Type");

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let msg = String::from_utf8_lossy(&output.stderr);
                if msg.contains("No session") {
                    return Ok(false);
                }
                error!(
                    "Couldn't get session: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else if output.status.success() {
                return Ok(true);
            }
        }
        Err(err) => error!("Couldn't get session: {}", err),
    }
    Ok(false)
}

fn get_session_type(session_id: &str) -> Result<SessionType, SessionError> {
    //loginctl show-session 2 --property Type
    let mut cmd = Command::new("loginctl");
    cmd.arg("show-session");
    cmd.arg(session_id);
    cmd.arg("--property");
    cmd.arg("Type");
    cmd.arg("--property");
    cmd.arg("Class");

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let msg = String::from_utf8_lossy(&output.stderr);
                if msg.contains("No session") {
                    return Err(SessionError::NoSession(session_id.into()));
                }
                error!(
                    "Couldn't get session: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else if output.status.success() {
                let what = String::from_utf8_lossy(&output.stdout);
                let mut stype = SessionType::TTY;
                let mut user = false;
                for line in what.lines() {
                    if let Some(is_it) = line.split("=").last() {
                        match is_it.trim() {
                            "user" => user = true,
                            "wayland" => stype = SessionType::Wayland,
                            "x11" => stype = SessionType::X11,
                            "tty" => stype = SessionType::TTY,
                            _ => return Err(SessionError::NoSession(session_id.into())),
                        }
                    }
                }
                if user {
                    return Ok(stype);
                }
            }
        }
        Err(err) => error!("Couldn't get session: {}", err),
    }
    Err(SessionError::NoSession(session_id.into()))
}

use std::fmt;

#[derive(Debug)]
pub enum SessionError {
    NoSession(String),
    NoSessions,
    Command(String, std::io::Error),
}

impl fmt::Display for SessionError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::NoSession(id) => write!(f, "Session {} not active", id),
            SessionError::NoSessions => write!(f, "No active sessions"),
            SessionError::Command(func, error) => {
                write!(f, "Command exec error: {}: {}", func, error)
            }
        }
    }
}

impl std::error::Error for SessionError {}

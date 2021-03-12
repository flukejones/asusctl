use serde::{Deserialize, Serialize};

use zvariant_derive::Type;
use zvariant::OwnedObjectPath;

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    sid: String,
    /// User ID
    uid: u32,
    /// Name of session user
    user: String,
    seat: String,
    path: OwnedObjectPath,
}

impl SessionInfo {
    pub fn sid(&self) -> &str {
        &self.sid
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn seat(&self) -> &str {
        &self.seat
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct Seat {
    id: String,
    /// Name of session user
    path: OwnedObjectPath,
}

impl Seat {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct User {
    uid: u32,
    name: String,
    /// Name of session user
    path: OwnedObjectPath,
}

impl User {
    pub fn uid(&self) -> u32 {
        self.uid
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }
}
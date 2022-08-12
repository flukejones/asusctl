/// Hardcoded layout. Was used to generate a toml default
pub mod g513;
/// Hardcoded layout. Was used to generate a toml default
pub mod ga401;
/// Hardcoded layout. Was used to generate a toml default
pub mod gx502;

use crate::{error::Error, keys::Key};
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Read, path::Path, slice::Iter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyLayout {
    /// A series of board names that this layout can be used for. The board names
    /// stored with the layout can be globbed, e.g, GA401 will match all of the
    /// GA401I and GA401Q range variants.
    ///
    /// `/sys/class/dmi/id/board_name`
    matches: Vec<String>,
    locale: String,
    rows: Vec<KeyRow>,
}

impl KeyLayout {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut buf = String::new();
        let read_len = file.read_to_string(&mut buf)?;
        if read_len == 0 {
            return Err(Error::Io(std::io::ErrorKind::InvalidData.into()));
        } else {
            return Ok(toml::from_str::<Self>(&buf)?);
        }
    }

    pub fn matches(&self, board_name: &str) -> bool {
        let board = board_name.to_ascii_uppercase();
        for tmp in self.matches.iter() {
            if board.contains(tmp.as_str()) {
                return true;
            }
        }
        false
    }

    pub fn rows(&self) -> Iter<KeyRow> {
        self.rows.iter()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyRow {
    height: f32,
    row: Vec<Key>,
}

impl KeyRow {
    pub fn new(height: f32, row: Vec<Key>) -> Self {
        Self { height, row }
    }

    pub fn row(&self) -> Iter<Key> {
        self.row.iter()
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

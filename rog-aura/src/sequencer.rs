use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde_derive::{Deserialize, Serialize};

use crate::error::Error;

/// All the possible AniMe actions that can be used. The enum is intended to be
/// used in a array allowing the user to cycle through a series of actions.
#[derive(Debug, Deserialize, Serialize)]
pub enum ActionData {
    Static,
}

/// An optimised precomputed set of actions that the user can cycle through
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Sequences(Vec<ActionData>);

impl Sequences {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Use a base `AnimeAction` to generate the precomputed data and insert in to
    /// the run buffer
    #[inline]
    pub fn insert(&mut self, index: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Remove an item at this position from the run buffer. If the `index` supplied
    /// is not in range then `None` is returned, otherwise the `ActionData` at that location
    /// is yeeted and returned.
    #[inline]
    pub fn remove_item(&mut self, index: usize) -> Option<ActionData> {
        if index < self.0.len() {
            return Some(self.0.remove(index));
        }
        None
    }

    pub fn iter(&self) -> ActionIterator {
        ActionIterator {
            actions: &self,
            next_idx: 0,
        }
    }
}

/// Iteractor helper for iterating over all the actions in `Sequences`
pub struct ActionIterator<'a> {
    actions: &'a Sequences,
    next_idx: usize,
}

impl<'a> Iterator for ActionIterator<'a> {
    type Item = &'a ActionData;

    #[inline]
    fn next(&mut self) -> Option<&'a ActionData> {
        if self.next_idx == self.actions.0.len() {
            self.next_idx = 0;
            return None;
        }

        let current = self.next_idx;
        self.next_idx += 1;

        Some(&self.actions.0[current])
    }
}

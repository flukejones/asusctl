use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use glam::Vec2;
use serde_derive::{Deserialize, Serialize};

use crate::{error::AnimeError, AnimeDataBuffer, AnimeGif, AnimeImage, AnimTime};

/// All the possible AniMe actions that can be used. This enum is intended to be
/// a helper for loading up `ActionData`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AnimeAction {
    /// Full gif sequence. Immutable.
    AsusAnimation {
        file: PathBuf,
        time: AnimTime,
        brightness: f32,
    },
    /// Basic image, can have properties changed
    ImageAnimation {
        file: PathBuf,
        scale: f32,
        angle: f32,
        translation: Vec2,
        time: AnimTime,
        brightness: f32,
    },
    Image {
        file: PathBuf,
        scale: f32,
        angle: f32,
        translation: Vec2,
        brightness: f32,
    },
    /// A pause to be used between sequences
    Pause(Duration),
}

/// All the possible AniMe actions that can be used. The enum is intended to be
/// used in a array allowing the user to cycle through a series of actions.
#[derive(Debug, Deserialize, Serialize)]
pub enum ActionData {
    /// Full gif sequence. Immutable.
    Animation(AnimeGif),
    /// Basic image, can have properties changed and image updated via those properties
    Image(Box<AnimeDataBuffer>),
    /// A pause to be used between sequences
    Pause(Duration),
    /// Placeholder
    AudioEq,
    /// Placeholder
    SystemInfo,
    /// Placeholder
    TimeDate,
    /// Placeholder
    Matrix,
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
    pub fn insert(&mut self, index: usize, action: &AnimeAction) -> Result<(), AnimeError> {
        match action {
            AnimeAction::AsusAnimation {
                file,
                time: duration,
                brightness,
            } => self.insert_asus_gif(index, &file, *duration, *brightness)?,
            AnimeAction::ImageAnimation {
                file,
                scale,
                angle,
                translation,
                time: duration,
                brightness,
            } => self.insert_image_gif(
                index,
                &file,
                *scale,
                *angle,
                *translation,
                *duration,
                *brightness,
            )?,
            AnimeAction::Image {
                file,
                scale,
                angle,
                translation,
                brightness,
            } => self.insert_png(index, &file, *scale, *angle, *translation, *brightness)?,
            AnimeAction::Pause(duration) => self.insert_pause(index, *duration),
        };
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

    fn insert_asus_gif(
        &mut self,
        mut index: usize,
        file: &Path,
        duration: AnimTime,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        if index > self.0.len() {
            index = self.0.len() - 1;
        }
        let frames = AnimeGif::create_diagonal_gif(file, duration, brightness)?;
        self.0.insert(index, ActionData::Animation(frames));
        Ok(())
    }

    fn insert_png(
        &mut self,
        mut index: usize,
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        if index > self.0.len() {
            index = self.0.len() - 1;
        }
        let image = AnimeImage::from_png(file, scale, angle, translation, brightness)?;
        let data = <AnimeDataBuffer>::from(&image);
        self.0.insert(index, ActionData::Image(Box::new(data)));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_image_gif(
        &mut self,
        mut index: usize,
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        duration: AnimTime,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        if index > self.0.len() {
            index = self.0.len() - 1;
        }
        let frames =
            AnimeGif::create_png_gif(file, scale, angle, translation, duration, brightness)?;
        self.0.insert(index, ActionData::Animation(frames));
        Ok(())
    }

    fn insert_pause(&mut self, mut index: usize, duration: Duration) {
        if index > self.0.len() {
            index = self.0.len() - 1;
        }
        self.0.insert(index, ActionData::Pause(duration));
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

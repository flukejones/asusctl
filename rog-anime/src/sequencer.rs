use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use glam::Vec2;
use serde_derive::{Deserialize, Serialize};

use crate::{error::AnimeError, AniMeDataBuffer, AniMeGif, AniMeImage, AnimTime};

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

///
#[derive(Debug, Deserialize, Serialize)]
pub enum ActionData {
    /// Full gif sequence. Immutable.
    Animation(AniMeGif),
    /// Basic image, can have properties changed and image updated via those properties
    Image(Box<AniMeDataBuffer>),
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

/// An optimised precomputed set of actions
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Sequences(Vec<ActionData>);

impl Sequences {
    pub fn new() -> Self {
        Self(Vec::new())
    }

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
        let frames = AniMeGif::create_diagonal_gif(file, duration, brightness)?;
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
        let image = AniMeImage::from_png(file, scale, angle, translation, brightness)?;
        let data = <AniMeDataBuffer>::from(&image);
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
            AniMeGif::create_png_gif(file, scale, angle, translation, duration, brightness)?;
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

pub struct ActionIterator<'a> {
    actions: &'a Sequences,
    next_idx: usize,
}

impl<'a> Iterator for ActionIterator<'a> {
    type Item = &'a ActionData;

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

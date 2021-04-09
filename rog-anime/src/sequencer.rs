use std::{path::Path, time::Duration};

use glam::Vec2;
use serde_derive::{Deserialize, Serialize};

use crate::{error::AnimeError, AniMeDataBuffer, AniMeGif, AniMeImage};

///
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
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
pub struct Sequences(Vec<Action>);

impl Sequences {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_asus_gif(
        &mut self,
        file: &Path,
        duration: Option<Duration>,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        let frames = AniMeGif::create_diagonal_gif(file, duration, brightness)?;
        self.0.push(Action::Animation(frames));
        Ok(())
    }

    pub fn add_png(
        &mut self,
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        let image = AniMeImage::from_png(file, scale, angle, translation, brightness)?;
        let data = <AniMeDataBuffer>::from(&image);
        self.0.push(Action::Image(Box::new(data)));
        Ok(())
    }

    pub fn add_image_gif(
        &mut self,
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        duration: Option<Duration>,
        brightness: f32,
    ) -> Result<(), AnimeError> {
        let frames =
            AniMeGif::create_png_gif(file, scale, angle, translation, duration, brightness)?;
        self.0.push(Action::Animation(frames));
        Ok(())
    }

    pub fn add_pause(&mut self, duration: Duration) -> Result<(), AnimeError> {
        self.0.push(Action::Pause(duration));
        Ok(())
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
    type Item = &'a Action;

    fn next(&mut self) -> Option<&'a Action> {
        if self.next_idx == self.actions.0.len() {
            self.next_idx = 0;
            return None;
        }

        let current = self.next_idx;
        self.next_idx += 1;

        Some(&self.actions.0[current])
    }
}

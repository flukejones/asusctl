use serde_derive::{Deserialize, Serialize};
/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod anime_data;
use std::{path::Path, time::Duration};

pub use anime_data::*;

/// Useful for specialised effects that required a grid of data
mod anime_grid;
pub use anime_grid::*;

/// Transform a PNG image for displaying on AniMe matrix display
mod anime_image;
pub use anime_image::*;

mod anime_diagonal;
pub use anime_diagonal::*;

mod anime_gif;
pub use anime_gif::*;
use error::AnimeError;

/// Base errors that are possible
pub mod error;

// TODO: make schema to rebuild the full sequence without requiring saving the actual
//  packet data

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AniMeBlock {
    /// Full gif sequence. Immutable.
    Animation(AniMeGif),
    /// Basic image, can have properties changed
    Image(Box<AniMeDataBuffer>),
    /// A pause to be used between sequences
    Pause(Duration),
}

impl AniMeBlock {
    pub fn asus_gif(file: &Path, brightness: f32) -> Result<Self, AnimeError> {
        let frames = AniMeGif::create_diagonal_gif(file, brightness)?;
        Ok(Self::Animation(frames))
    }

    pub fn png(
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        brightness: f32,
    ) -> Result<Self, AnimeError> {
        let image = AniMeImage::from_png(file, scale, angle, translation, brightness)?;
        let data = <AniMeDataBuffer>::from(&image);
        Ok(Self::Image(Box::new(data)))
    }

    pub fn image_gif(
        file: &Path,
        scale: f32,
        angle: f32,
        translation: Vec2,
        brightness: f32,
    ) -> Result<Self, AnimeError> {
        let frames = AniMeGif::create_png_gif(file, scale, angle, translation, brightness)?;
        Ok(Self::Animation(frames))
    }

    pub fn get_animation(&self) -> Option<&AniMeGif> {
        match self {
            AniMeBlock::Animation(anim) => Some(anim),
            _ => None,
        }
    }

    pub fn get_image(&self) -> Option<&AniMeDataBuffer> {
        match self {
            AniMeBlock::Image(image) => Some(image),
            _ => None,
        }
    }

    pub fn get_pause(&self) -> Option<Duration> {
        match self {
            AniMeBlock::Pause(pause) => Some(*pause),
            _ => None,
        }
    }
}

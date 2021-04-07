use std::{
    fs::{create_dir, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

use rog_anime::{Sequences, Vec2};
use serde_derive::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct UserConfig {
    anime: Vec<AnimeAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AnimeAction {
    /// Full gif sequence. Immutable.
    AsusAnimation {
        file: PathBuf,
        duration: Option<Duration>,
        brightness: f32,
    },
    /// Basic image, can have properties changed
    ImageAnimation {
        file: PathBuf,
        scale: f32,
        angle: f32,
        translation: Vec2,
        duration: Option<Duration>,
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

impl UserConfig {
    pub fn new() -> Self {
        Self {
            anime: vec![
                AnimeAction::AsusAnimation {
                    file: "/usr/share/asusd/anime/asus/rog/Sunset.gif".into(),
                    brightness: 0.5,
                    duration: None,
                },
                AnimeAction::ImageAnimation {
                    file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                    scale: 0.9,
                    angle: 0.65,
                    translation: Vec2::default(),
                    brightness: 0.5,
                    duration: Some(Duration::from_secs(5)),
                },
                AnimeAction::Image {
                    file: "/usr/share/asusd/anime/custom/rust.png".into(),
                    scale: 1.0,
                    angle: 0.0,
                    translation: Vec2::default(),
                    brightness: 0.6,
                },
                AnimeAction::Pause(Duration::from_secs(6)),
                AnimeAction::ImageAnimation {
                    file: "/usr/share/asusd/anime/custom/sonic-wait.gif".into(),
                    scale: 0.9,
                    angle: 0.0,
                    translation: Vec2::new(3.0, 2.0),
                    brightness: 0.5,
                    duration: None,
                },
            ],
        }
    }

    pub fn load_config(&mut self) -> Result<(), Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push("rog");
        if !path.exists() {
            create_dir(path.clone())?;
        }

        path.push("rog-user.cfg");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let mut buf = String::new();

        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                let json = serde_json::to_string_pretty(&self).unwrap();
                file.write_all(json.as_bytes())?;
            } else if let Ok(data) = serde_json::from_str::<UserConfig>(&buf) {
                self.anime = data.anime;
                return Ok(());
            }
        }
        Ok(())
        //Err(Error::ConfigLoadFail)
    }

    pub fn create_anime(&self) -> Result<Sequences, Error> {
        let mut seq = Sequences::new();

        for anime in self.anime.iter() {
            match anime {
                AnimeAction::AsusAnimation {
                    file,
                    duration,
                    brightness,
                } => seq.add_asus_gif(&file, *duration, *brightness)?,
                AnimeAction::ImageAnimation {
                    file,
                    scale,
                    angle,
                    translation,
                    duration,
                    brightness,
                } => {
                    seq.add_image_gif(&file, *scale, *angle, *translation, *duration, *brightness)?
                }
                AnimeAction::Image {
                    file,
                    scale,
                    angle,
                    translation,
                    brightness,
                } => seq.add_png(&file, *scale, *angle, *translation, *brightness)?,
                AnimeAction::Pause(duration) => seq.add_pause(*duration)?,
            }
        }

        Ok(seq)
    }
}

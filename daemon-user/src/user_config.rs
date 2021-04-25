use std::{
    fs::{create_dir, OpenOptions},
    io::{Read, Write},
    time::Duration,
};

use rog_anime::{AnimTime, AnimeAction, Sequences, Vec2};
use serde_derive::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAnimeConfig {
    pub name: String,
    pub anime: Vec<AnimeAction>,
}

impl UserAnimeConfig {
    pub fn create_anime(&self) -> Result<Sequences, Error> {
        let mut seq = Sequences::new();

        for (idx, action) in self.anime.iter().enumerate() {
            seq.insert(idx, action)?;
        }

        Ok(seq)
    }

    pub fn write(&self) -> Result<(), Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push("rog");
        if !path.exists() {
            create_dir(path.clone())?;
        }
        let name = self.name.clone();
        path.push(name + ".cfg");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let json = serde_json::to_string_pretty(&self).unwrap();
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_config(name: String) -> Result<UserAnimeConfig, Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push("rog");
        if !path.exists() {
            create_dir(path.clone())?;
        }

        path.push(name + ".cfg");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let mut buf = String::new();

        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                let default = UserAnimeConfig::default();
                let json = serde_json::to_string_pretty(&default).unwrap();
                file.write_all(json.as_bytes())?;
                return Ok(default);
            } else if let Ok(data) = serde_json::from_str::<UserAnimeConfig>(&buf) {
                return Ok(data);
            }
        }
        Err(Error::ConfigLoadFail)
    }
}

impl Default for UserAnimeConfig {
    fn default() -> Self {
        let x = Self {
            name: "default".to_string(),
            anime: vec![
                AnimeAction::AsusAnimation {
                    file: "/usr/share/asusd/anime/asus/rog/Sunset.gif".into(),
                    brightness: 0.5,
                    time: AnimTime::Cycles(1),
                },
                AnimeAction::ImageAnimation {
                    file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                    scale: 0.9,
                    angle: 0.65,
                    translation: Vec2::default(),
                    brightness: 0.5,
                    time: AnimTime::Time(Duration::from_secs(5)),
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
                    time: AnimTime::Cycles(2),
                },
            ],
        };
        println!("{}", serde_json::to_string_pretty(&x).unwrap());
        x
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct UserConfig {
    /// Name of active anime config file in the user config directory
    pub active_anime: String,
}

impl UserConfig {
    pub fn new() -> Self {
        Self {
            active_anime: "anime-default".to_string(),
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
                self.active_anime = data.active_anime;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn write(&self) -> Result<(), Error> {
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
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let json = serde_json::to_string_pretty(&self).unwrap();
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

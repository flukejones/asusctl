use std::{
    fs::{create_dir, OpenOptions},
    io::{Read, Write},
    time::Duration,
};

use rog_anime::{ActionLoader, AnimTime, AnimeType, Fade, Sequences, Vec2};
use rog_aura::{keys::Key, Breathe, Colour, Effect, Flicker, LedType, Speed, Static};
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};

use crate::error::Error;

pub trait ConfigLoadSave<T: DeserializeOwned + serde::Serialize> {
    fn name(&self) -> String;

    fn default_with_name(name: String) -> T;

    fn write(&self) -> Result<(), Error>
    where
        Self: serde::Serialize,
    {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push("rog");
        if !path.exists() {
            create_dir(path.clone())?;
        }
        let name = self.name();
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

    fn load(name: String) -> Result<T, Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push("rog");
        if !path.exists() {
            create_dir(path.clone())?;
        }

        path.push(name.clone() + ".cfg");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let mut buf = String::new();

        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                let default = Self::default_with_name(name);
                let json = serde_json::to_string_pretty(&default).unwrap();
                file.write_all(json.as_bytes())?;
                return Ok(default);
            } else if let Ok(data) = serde_json::from_str::<T>(&buf) {
                return Ok(data);
            }
        }
        Err(Error::ConfigLoadFail)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAnimeConfig {
    pub name: String,
    pub anime: Vec<ActionLoader>,
}

impl UserAnimeConfig {
    pub fn create(&self, anime_type: AnimeType) -> Result<Sequences, Error> {
        let mut seq = Sequences::new(anime_type);

        for (idx, action) in self.anime.iter().enumerate() {
            seq.insert(idx, action)?;
        }

        Ok(seq)
    }
}

impl ConfigLoadSave<UserAnimeConfig> for UserAnimeConfig {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn default_with_name(name: String) -> Self {
        UserAnimeConfig {
            name,
            ..Default::default()
        }
    }
}

impl Default for UserAnimeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            anime: vec![
                ActionLoader::AsusImage {
                    file: "/usr/share/asusd/anime/custom/diagonal-template.png".into(),
                    brightness: 1.0,
                    time: AnimTime::Fade(Fade::new(
                        Duration::from_secs(2),
                        None,
                        Duration::from_secs(2),
                    )),
                },
                ActionLoader::AsusAnimation {
                    file: "/usr/share/asusd/anime/asus/rog/Sunset.gif".into(),
                    brightness: 0.5,
                    time: AnimTime::Fade(Fade::new(
                        Duration::from_secs(6),
                        None,
                        Duration::from_secs(3),
                    )),
                },
                ActionLoader::ImageAnimation {
                    file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                    scale: 0.9,
                    angle: 0.65,
                    translation: Vec2::default(),
                    brightness: 0.5,
                    time: AnimTime::Fade(Fade::new(
                        Duration::from_secs(2),
                        Some(Duration::from_secs(2)),
                        Duration::from_secs(2),
                    )),
                },
                ActionLoader::Image {
                    file: "/usr/share/asusd/anime/custom/rust.png".into(),
                    scale: 1.0,
                    angle: 0.0,
                    translation: Vec2::default(),
                    time: AnimTime::Fade(Fade::new(
                        Duration::from_secs(2),
                        Some(Duration::from_secs(1)),
                        Duration::from_secs(2),
                    )),
                    brightness: 0.6,
                },
                ActionLoader::Pause(Duration::from_secs(1)),
                ActionLoader::ImageAnimation {
                    file: "/usr/share/asusd/anime/custom/sonic-wait.gif".into(),
                    scale: 0.9,
                    angle: 0.0,
                    translation: Vec2::new(3.0, 2.0),
                    brightness: 0.5,
                    time: AnimTime::Count(2),
                },
            ],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAuraConfig {
    pub name: String,
    pub aura: rog_aura::Sequences,
}

impl ConfigLoadSave<UserAuraConfig> for UserAuraConfig {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn default_with_name(name: String) -> Self {
        UserAuraConfig {
            name,
            ..Default::default()
        }
    }
}

impl Default for UserAuraConfig {
    fn default() -> Self {
        let mut seq = rog_aura::Sequences::new();
        let mut key = Effect::Breathe(Breathe::new(
            LedType::Key(Key::W),
            Colour(255, 0, 20),
            Colour(20, 255, 0),
            Speed::Low,
        ));

        seq.push(key.clone());
        key.set_led_type(LedType::Key(Key::A));
        seq.push(key.clone());
        key.set_led_type(LedType::Key(Key::S));
        seq.push(key.clone());
        key.set_led_type(LedType::Key(Key::D));
        seq.push(key);

        let key = Effect::Breathe(Breathe::new(
            LedType::Key(Key::F),
            Colour(255, 0, 0),
            Colour(255, 0, 0),
            Speed::High,
        ));
        seq.push(key);

        let mut key = Effect::Static(Static::new(LedType::Key(Key::RCtrl), Colour(0, 0, 255)));
        seq.push(key.clone());
        key.set_led_type(LedType::Key(Key::LCtrl));
        seq.push(key.clone());
        key.set_led_type(LedType::Key(Key::Esc));
        seq.push(key);

        let key = Effect::Flicker(Flicker::new(
            LedType::Key(Key::N9),
            Colour(0, 0, 255),
            80,
            40,
        ));
        seq.push(key);

        Self {
            name: "default".to_owned(),
            aura: seq,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct UserConfig {
    /// Name of active anime config file in the user config directory
    pub active_anime: Option<String>,
    /// Name of active aura config file in the user config directory
    pub active_aura: Option<String>,
}

impl UserConfig {
    pub fn new() -> Self {
        Self {
            active_anime: Some("anime-default".to_owned()),
            active_aura: Some("aura-default".to_owned()),
        }
    }

    pub fn load(&mut self) -> Result<(), Error> {
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
                self.active_aura = data.active_aura;
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

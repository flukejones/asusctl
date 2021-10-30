use rog_anime::error::AnimeError;
use rog_anime::{ActionData, ActionLoader, AnimTime, Fade, Sequences, Vec2};
use rog_dbus::RogDbusClient;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use std::{sync::Arc, thread::sleep, time::Instant};
use zbus::dbus_interface;
use zvariant::ObjectPath;
use zvariant_derive::Type;

use crate::{error::Error, user_config::UserAnimeConfig};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct Timer {
    type_of: TimeType,
    /// If time type is Timer then this is milliseonds, otherwise it is animation loop count
    count: u64,
    /// Used only for `TimeType::Timer`, milliseonds to fade the image in for
    fade_in: Option<u64>,
    /// Used only for `TimeType::Timer`, milliseonds to fade the image out for
    fade_out: Option<u64>,
}

impl From<Timer> for AnimTime {
    fn from(time: Timer) -> Self {
        match time.type_of {
            TimeType::Timer => {
                if time.fade_in.is_some() || time.fade_out.is_some() {
                    let fade_in = time
                        .fade_in
                        .map_or(Duration::from_secs(0), Duration::from_millis);
                    let fade_out = time
                        .fade_out
                        .map_or(Duration::from_secs(0), Duration::from_millis);
                    let show_for = if time.count != 0 {
                        Some(Duration::from_millis(time.count))
                    } else {
                        None
                    };
                    AnimTime::Fade(Fade::new(fade_in, show_for, fade_out))
                } else {
                    AnimTime::Time(Duration::from_millis(time.count))
                }
            }
            TimeType::Count => AnimTime::Count(time.count as u32),
            TimeType::Infinite => AnimTime::Infinite,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub enum TimeType {
    Timer,
    Count,
    Infinite,
}

/// The inner object exists to allow the zbus proxy to share it with a runner thread
/// and a zbus server behind `Arc<Mutex<T>>`
pub struct CtrlAnimeInner<'a> {
    sequences: Sequences,
    client: RogDbusClient<'a>,
    do_early_return: Arc<AtomicBool>,
}

impl<'a> CtrlAnimeInner<'static> {
    pub fn new(
        sequences: Sequences,
        client: RogDbusClient<'static>,
        do_early_return: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        Ok(Self {
            sequences,
            client,
            do_early_return,
        })
    }
    /// To be called on each main loop iteration to pump out commands to the anime
    pub fn run(&'a self) -> Result<(), Error> {
        if self.do_early_return.load(Ordering::SeqCst) {
            return Ok(());
        }

        for action in self.sequences.iter() {
            match action {
                ActionData::Animation(frames) => {
                    rog_anime::run_animation(frames, self.do_early_return.clone(), &|output| {
                        self.client
                            .proxies()
                            .anime()
                            .write(output)
                            .map_err(|e| AnimeError::Dbus(format!("{}", e)))
                    })?;
                }
                ActionData::Image(image) => {
                    self.client
                        .proxies()
                        .anime()
                        .write(image.as_ref().clone())
                        .ok();
                }
                ActionData::Pause(duration) => {
                    let start = Instant::now();
                    'pause: loop {
                        if self.do_early_return.load(Ordering::SeqCst) {
                            return Ok(());
                        }
                        if Instant::now().duration_since(start) > *duration {
                            break 'pause;
                        }
                        sleep(Duration::from_millis(1));
                    }
                }
                ActionData::AudioEq => {}
                ActionData::SystemInfo => {}
                ActionData::TimeDate => {}
                ActionData::Matrix => {}
            }
        }

        Ok(())
    }
}

pub struct CtrlAnime<'a> {
    config: Arc<Mutex<UserAnimeConfig>>,
    client: RogDbusClient<'a>,
    inner: Arc<Mutex<CtrlAnimeInner<'a>>>,
    /// Must be the same Atomic as in CtrlAnimeInner
    inner_early_return: Arc<AtomicBool>,
}

impl<'a> CtrlAnime<'static> {
    pub fn new(
        config: Arc<Mutex<UserAnimeConfig>>,
        inner: Arc<Mutex<CtrlAnimeInner<'static>>>,
        client: RogDbusClient<'static>,
        inner_early_return: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        Ok(CtrlAnime {
            config,
            client,
            inner,
            inner_early_return,
        })
    }

    pub fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Anime"),
                self,
            )
            .map_err(|err| {
                println!("CtrlAnime: add_to_server {}", err);
                err
            })
            .ok();
    }
}

// The pattern for a zbus method is:
// - Get config lock if required
// - Set inner_early_return to stop the inner run loop temporarily
// - Do actions
// - Write config if required
// - Unset inner_early_return
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnime<'static> {
    pub fn insert_asus_gif(
        &mut self,
        index: u32,
        file: String,
        time: Timer,
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let time: AnimTime = time.into();
            let file = Path::new(&file);
            let action = ActionLoader::AsusAnimation {
                file: file.into(),
                brightness,
                time,
            };

            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller
                    .sequences
                    .insert(index as usize, &action)
                    .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
            }
            config.anime.push(action);
            config.write()?;

            let json = serde_json::to_string_pretty(&*config).expect("Parse config to JSON failed");

            // Release the inner run loop again
            self.inner_early_return.store(false, Ordering::SeqCst);
            return Ok(json);
        }
        Err(zbus::fdo::Error::Failed("UserConfig lock fail".into()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_image_gif(
        &mut self,
        index: u32,
        file: String,
        scale: f32,
        angle: f32,
        xy: (f32, f32),
        time: Timer,
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let time: AnimTime = time.into();
            let file = Path::new(&file);
            let translation = Vec2::new(xy.0, xy.1);
            let action = ActionLoader::ImageAnimation {
                file: file.into(),
                scale,
                angle,
                translation,
                brightness,
                time,
            };

            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller
                    .sequences
                    .insert(index as usize, &action)
                    .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
            }
            config.anime.push(action);
            config.write()?;

            let json =
                serde_json::to_string_pretty(&*config.anime).expect("Parse config to JSON failed");

            // Release the inner run loop again
            self.inner_early_return.store(false, Ordering::SeqCst);
            return Ok(json);
        }
        Err(zbus::fdo::Error::Failed("UserConfig lock fail".into()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_image(
        &mut self,
        index: u32,
        file: String,
        scale: f32,
        angle: f32,
        xy: (f32, f32),
        time: Timer,
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let file = Path::new(&file);
            let time = time.into();
            let action = ActionLoader::Image {
                file: file.into(),
                scale,
                angle,
                translation: Vec2::new(xy.0, xy.1),
                brightness,
                time,
            };

            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller
                    .sequences
                    .insert(index as usize, &action)
                    .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
            }
            config.anime.push(action);
            config.write()?;

            let json =
                serde_json::to_string_pretty(&*config.anime).expect("Parse config to JSON failed");

            // Release the inner run loop again
            self.inner_early_return.store(false, Ordering::SeqCst);
            return Ok(json);
        }
        Err(zbus::fdo::Error::Failed("UserConfig lock fail".into()))
    }

    pub fn insert_pause(&mut self, index: u32, millis: u64) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let action = ActionLoader::Pause(Duration::from_millis(millis));
            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller
                    .sequences
                    .insert(index as usize, &action)
                    .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
            }
            config.anime.push(action);
            config.write()?;

            let json =
                serde_json::to_string_pretty(&*config.anime).expect("Parse config to JSON failed");

            // Release the inner run loop again
            self.inner_early_return.store(false, Ordering::SeqCst);
            return Ok(json);
        }
        Err(zbus::fdo::Error::Failed("UserConfig lock fail".into()))
    }

    pub fn remove_item(&mut self, index: u32) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller.sequences.remove_item(index as usize);
            }
            if (index as usize) < config.anime.len() {
                config.anime.remove(index as usize);
            }
            config.write()?;

            let json =
                serde_json::to_string_pretty(&*config.anime).expect("Parse config to JSON failed");

            // Release the inner run loop again
            self.inner_early_return.store(false, Ordering::SeqCst);
            return Ok(json);
        }
        Err(zbus::fdo::Error::Failed("UserConfig lock fail".into()))
    }

    pub fn set_state(&mut self, on: bool) -> zbus::fdo::Result<()> {
        // Operations here need to be in specific order
        if on {
            self.client.proxies().anime().set_on_off(on)?;
            // Let the inner loop run
            self.inner_early_return.store(false, Ordering::SeqCst);
        } else {
            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);
            self.client.proxies().anime().set_on_off(on)?;
        }
        Ok(())
    }
}

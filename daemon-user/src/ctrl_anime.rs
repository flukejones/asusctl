use rog_anime::{ActionData, AnimTime, AnimeAction, Sequences, Vec2};
use rog_dbus::AuraDbusClient;
//use crate::dbus::DbusEvents;
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

use crate::{error::Error, user_config::UserConfig};

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
    client: AuraDbusClient<'a>,
    do_early_return: &'a AtomicBool,
}

impl<'a> CtrlAnimeInner<'static> {
    pub fn new(
        sequences: Sequences,
        client: AuraDbusClient<'static>,
        do_early_return: &'static AtomicBool,
    ) -> Result<Self, Error> {
        Ok(Self {
            sequences,
            client,
            do_early_return,
        })
    }
    /// To be called on each main loop iteration to pump out commands to the anime
    pub fn run(&self) -> Result<(), Error> {
        if self.do_early_return.load(Ordering::SeqCst) {
            return Ok(());
        }

        for action in self.sequences.iter() {
            match action {
                ActionData::Animation(frames) => {
                    let mut count = 0;
                    let start = Instant::now();
                    'animation: loop {
                        for frame in frames.frames() {
                            if self.do_early_return.load(Ordering::SeqCst) {
                                return Ok(());
                            }
                            self.client
                                .proxies()
                                .anime()
                                .write(frame.frame().clone())
                                .unwrap();
                            if let AnimTime::Time(time) = frames.duration() {
                                if Instant::now().duration_since(start) > time {
                                    break 'animation;
                                }
                            }
                            sleep(frame.delay());
                        }
                        if let AnimTime::Cycles(times) = frames.duration() {
                            count += 1;
                            if count >= times {
                                break 'animation;
                            }
                        }
                    }
                }
                ActionData::Image(image) => {
                    self.client
                        .proxies()
                        .anime()
                        .write(image.as_ref().clone())
                        .unwrap();
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
    config: Arc<Mutex<UserConfig>>,
    client: AuraDbusClient<'a>,
    inner: Arc<Mutex<CtrlAnimeInner<'a>>>,
    /// Must be the same Atomic as in CtrlAnimeInner
    inner_early_return: &'a AtomicBool,
}

impl<'a> CtrlAnime<'static> {
    pub fn new(
        config: Arc<Mutex<UserConfig>>,
        inner: Arc<Mutex<CtrlAnimeInner<'static>>>,
        client: AuraDbusClient<'static>,
        inner_early_return: &'static AtomicBool,
    ) -> Result<Self, Error> {
        Ok(CtrlAnime {
            config,
            inner,
            client,
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
        time: TimeType,
        count: u32,
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let time: AnimTime = match time {
                TimeType::Timer => AnimTime::Time(Duration::from_millis(count as u64)),
                TimeType::Count => AnimTime::Cycles(count),
                TimeType::Infinite => AnimTime::Infinite,
            };
            let file = Path::new(&file);
            let action = AnimeAction::AsusAnimation {
                file: file.into(),
                brightness,
                time,
            };

            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller.sequences.insert(index as usize, &action)?;
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
        time: TimeType,
        count: u32,
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let time: AnimTime = match time {
                TimeType::Timer => AnimTime::Time(Duration::from_millis(count as u64)),
                TimeType::Count => AnimTime::Cycles(count),
                TimeType::Infinite => AnimTime::Infinite,
            };
            let file = Path::new(&file);
            let translation = Vec2::new(xy.0, xy.1);
            let action = AnimeAction::ImageAnimation {
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
                controller.sequences.insert(index as usize, &action)?;
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

    pub fn insert_image(
        &mut self,
        index: u32,
        file: String,
        scale: f32,
        angle: f32,
        xy: (f32, f32),
        brightness: f32,
    ) -> zbus::fdo::Result<String> {
        if let Ok(mut config) = self.config.try_lock() {
            let file = Path::new(&file);
            let action = AnimeAction::Image {
                file: file.into(),
                scale,
                angle,
                translation: Vec2::new(xy.0, xy.1),
                brightness,
            };

            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller.sequences.insert(index as usize, &action)?;
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
            let action = AnimeAction::Pause(Duration::from_millis(millis));
            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);

            if let Ok(mut controller) = self.inner.lock() {
                controller.sequences.insert(index as usize, &action)?;
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
            self.client.proxies().anime().toggle_on(on)?;
            // Let the inner loop run
            self.inner_early_return.store(false, Ordering::SeqCst);
        } else {
            // Must make the inner run loop return early
            self.inner_early_return.store(true, Ordering::SeqCst);
            self.client.proxies().anime().toggle_on(on)?;
        }
        Ok(())
    }
}

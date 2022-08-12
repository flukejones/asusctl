pub mod config;
pub mod zbus;

use ::zbus::Connection;
use async_trait::async_trait;
use log::{error, info, warn};
use logind_zbus::manager::ManagerProxy;
use rog_anime::{
    error::AnimeError,
    usb::{
        get_anime_type, pkt_for_apply, pkt_for_flush, pkt_for_set_boot, pkt_for_set_on,
        pkts_for_init,
    },
    ActionData, AnimeDataBuffer, AnimePacketType, AnimeType,
};
use rog_platform::{hid_raw::HidRaw, usb_raw::USBRaw};
use rog_supported::AnimeSupportedFunctions;
use smol::{stream::StreamExt, Executor};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    convert::TryFrom,
    error::Error,
    sync::{Arc, Mutex, MutexGuard},
    thread::sleep,
};

use crate::{error::RogError, GetSupported};

use self::config::{AnimeConfig, AnimeConfigCached};

impl GetSupported for CtrlAnime {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(HidRaw::new("193b").is_ok())
    }
}

pub struct CtrlAnime {
    node: USBRaw,
    anime_type: AnimeType,
    cache: AnimeConfigCached,
    config: AnimeConfig,
    // set to force thread to exit
    thread_exit: Arc<AtomicBool>,
    // Set to false when the thread exits
    thread_running: Arc<AtomicBool>,
}

impl CtrlAnime {
    #[inline]
    pub fn new(config: AnimeConfig) -> Result<CtrlAnime, Box<dyn Error>> {
        let node = USBRaw::new(0x193b)?;
        let anime_type = get_anime_type()?;

        info!("Device has an AniMe Matrix display");
        let mut cache = AnimeConfigCached::default();
        cache.init_from_config(&config, anime_type)?;

        let ctrl = CtrlAnime {
            node,
            anime_type,
            cache,
            config,
            thread_exit: Arc::new(AtomicBool::new(false)),
            thread_running: Arc::new(AtomicBool::new(false)),
        };
        ctrl.do_initialization()?;

        Ok(ctrl)
    }
    // let device = CtrlAnime::get_device(0x0b05, 0x193b)?;

    /// Start an action thread. This is classed as a singleton and there should be only
    /// one running - so the thread uses atomics to signal run/exit.
    ///
    /// Because this also writes to the usb device, other write tries (display only) *must*
    /// get the mutex lock and set the thread_exit atomic.
    fn run_thread(inner: Arc<Mutex<CtrlAnime>>, actions: Vec<ActionData>, mut once: bool) {
        if actions.is_empty() {
            warn!("AniMe system actions was empty");
            return;
        }

        // Loop rules:
        // - Lock the mutex **only when required**. That is, the lock must be held for the shortest duration possible.
        // - An AtomicBool used for thread exit should be checked in every loop, including nested

        // The only reason for this outer thread is to prevent blocking while waiting for the
        // next spawned thread to exit
        std::thread::Builder::new()
            .name("AniMe system thread start".into())
            .spawn(move || {
                info!("AniMe new system thread started");
                // Getting copies of these Atomics is done *in* the thread to ensure
                // we don't block other threads/main
                let thread_exit;
                let thread_running;
                let anime_type;
                loop {
                    if let Ok(lock) = inner.try_lock() {
                        thread_exit = lock.thread_exit.clone();
                        thread_running = lock.thread_running.clone();
                        anime_type = lock.anime_type;
                        break;
                    }
                }
                // First two loops are to ensure we *do* aquire a lock on the mutex
                // The reason the loop is required is because the USB writes can block
                // for up to 10ms. We can't fail to get the atomics.
                while thread_running.load(Ordering::SeqCst) {
                    // Make any running loop exit first
                    thread_exit.store(true, Ordering::SeqCst);
                }

                info!("AniMe no previous system thread running (now)");
                thread_exit.store(false, Ordering::SeqCst);

                'main: loop {
                    thread_running.store(true, Ordering::SeqCst);
                    for action in actions.iter() {
                        if thread_exit.load(Ordering::SeqCst) {
                            break 'main;
                        }
                        match action {
                            ActionData::Animation(frames) => {
                                if let Err(err) = rog_anime::run_animation(frames, &|frame| {
                                    if thread_exit.load(Ordering::Acquire) {
                                        info!("rog-anime: frame-loop was asked to exit");
                                        return Ok(true); // Do safe exit
                                    }
                                    inner
                                        .try_lock()
                                        .map(|lock| {
                                            lock.write_data_buffer(frame)
                                                .map_err(|err| {
                                                    warn!(
                                                        "rog_anime::run_animation:callback {}",
                                                        err
                                                    );
                                                })
                                                .ok();
                                            false // Don't exit yet
                                        })
                                        .map_err(|err| {
                                            warn!("rog_anime::run_animation:callback {}", err);
                                            AnimeError::NoFrames
                                        })
                                }) {
                                    warn!("rog_anime::run_animation:Animation {}", err);
                                    break 'main;
                                };
                            }
                            ActionData::Image(image) => {
                                once = false;
                                if let Ok(lock) = inner.try_lock() {
                                    lock.write_data_buffer(image.as_ref().clone())
                                        .map_err(|e| error!("{}", e))
                                        .ok();
                                }
                            }
                            ActionData::Pause(duration) => sleep(*duration),
                            ActionData::AudioEq => {}
                            ActionData::SystemInfo => {}
                            ActionData::TimeDate => {}
                            ActionData::Matrix => {}
                        }
                    }
                    if thread_exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
                    if once || actions.is_empty() {
                        break 'main;
                    }
                }
                // Clear the display on exit
                if let Ok(lock) = inner.try_lock() {
                    if let Ok(data) =
                        AnimeDataBuffer::from_vec(anime_type, vec![0u8; anime_type.data_length()])
                            .map_err(|e| error!("{}", e))
                    {
                        lock.write_data_buffer(data)
                            .map_err(|err| {
                                warn!("rog_anime::run_animation:callback {}", err);
                            })
                            .ok();
                    }
                }
                // Loop ended, set the atmonics
                thread_running.store(false, Ordering::SeqCst);
                info!("AniMe system thread exited");
            })
            .map(|err| info!("AniMe system thread: {:?}", err))
            .ok();
    }

    /// Write only a data packet. This will modify the leds brightness using the
    /// global brightness set in config.
    fn write_data_buffer(&self, mut buffer: AnimeDataBuffer) -> Result<(), RogError> {
        for led in buffer.data_mut().iter_mut() {
            let mut bright = *led as f32 * self.config.brightness;
            if bright > 254.0 {
                bright = 254.0;
            }
            *led = bright as u8;
        }
        let data = AnimePacketType::try_from(buffer)?;
        for row in data.iter() {
            self.node.write_bytes(row)?;
        }
        self.node.write_bytes(&pkt_for_flush())?;
        Ok(())
    }

    fn do_initialization(&self) -> Result<(), RogError> {
        let pkts = pkts_for_init();
        self.node.write_bytes(&pkts[0])?;
        self.node.write_bytes(&pkts[1])?;
        Ok(())
    }
}

pub struct CtrlAnimeTask {
    inner: Arc<Mutex<CtrlAnime>>,
}

impl CtrlAnimeTask {
    pub async fn new(inner: Arc<Mutex<CtrlAnime>>) -> CtrlAnimeTask {
        Self { inner }
    }
}

#[async_trait]
impl crate::CtrlTask for CtrlAnimeTask {
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let connection = Connection::system()
            .await
            .expect("CtrlAnimeTask could not create dbus connection");

        let manager = ManagerProxy::new(&connection)
            .await
            .expect("CtrlAnimeTask could not create ManagerProxy");

        let run_action =
            |start: bool, lock: MutexGuard<CtrlAnime>, inner: Arc<Mutex<CtrlAnime>>| {
                if start {
                    info!("CtrlAnimeTask running sleep animation");
                    CtrlAnime::run_thread(inner.clone(), lock.cache.shutdown.clone(), true);
                } else {
                    info!("CtrlAnimeTask running wake animation");
                    CtrlAnime::run_thread(inner.clone(), lock.cache.wake.clone(), true);
                }
            };

        let inner = self.inner.clone();
        executor
            .spawn(async move {
                if let Ok(notif) = manager.receive_prepare_for_sleep().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                // Loop is required to try an attempt to get the mutex *without* blocking
                                // other threads - it is possible to end up with deadlocks otherwise.
                                loop {
                                    if let Ok(lock) = inner.clone().try_lock() {
                                        run_action(args.start, lock, inner.clone());
                                        break;
                                    }
                                }
                            }
                        })
                        .await;
                }
            })
            .detach();

        let manager = ManagerProxy::new(&connection)
            .await
            .expect("CtrlAnimeTask could not create ManagerProxy");

        let inner = self.inner.clone();
        executor
            .spawn(async move {
                if let Ok(notif) = manager.receive_prepare_for_shutdown().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                loop {
                                    if let Ok(lock) = inner.clone().try_lock() {
                                        run_action(args.start, lock, inner.clone());
                                    }
                                }
                            }
                        })
                        .await;
                }
            })
            .detach();

        Ok(())
    }
}

pub struct CtrlAnimeReloader(pub Arc<Mutex<CtrlAnime>>);

impl crate::Reloadable for CtrlAnimeReloader {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(lock) = self.0.try_lock() {
            lock.node
                .write_bytes(&pkt_for_set_on(lock.config.awake_enabled))?;
            lock.node.write_bytes(&pkt_for_apply())?;
            lock.node
                .write_bytes(&pkt_for_set_boot(lock.config.boot_anim_enabled))?;
            lock.node.write_bytes(&pkt_for_apply())?;

            let action = lock.cache.boot.clone();
            CtrlAnime::run_thread(self.0.clone(), action, true);
        }
        Ok(())
    }
}

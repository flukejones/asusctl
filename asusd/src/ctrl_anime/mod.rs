pub mod config;
/// Implements `CtrlTask`, Reloadable, `ZbusRun`
pub mod trait_impls;

use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;

use ::zbus::export::futures_util::lock::Mutex;
use log::{error, info, warn};
use rog_anime::error::AnimeError;
use rog_anime::usb::{
    get_anime_type, pkt_flush, pkt_set_brightness, pkt_set_enable_display,
    pkt_set_enable_powersave_anim, pkts_for_init, Brightness,
};
use rog_anime::{ActionData, AnimeDataBuffer, AnimePacketType, AnimeType};
use rog_platform::hid_raw::HidRaw;
use rog_platform::usb_raw::USBRaw;

use self::config::{AnimeConfig, AnimeConfigCached};
use crate::error::RogError;

enum Node {
    Usb(USBRaw),
    Hid(HidRaw),
}

impl Node {
    pub fn write_bytes(&self, message: &[u8]) -> Result<(), RogError> {
        // TODO: map and pass on errors
        match self {
            Node::Usb(u) => {
                u.write_bytes(message).ok();
            }
            Node::Hid(h) => {
                h.write_bytes(message).ok();
            }
        }
        Ok(())
    }

    pub fn set_builtins_enabled(&self, enabled: bool, bright: Brightness) -> Result<(), RogError> {
        self.write_bytes(&pkt_set_enable_powersave_anim(enabled))?;
        self.write_bytes(&pkt_set_enable_display(enabled))?;
        self.write_bytes(&pkt_set_brightness(bright))?;
        self.write_bytes(&pkt_set_enable_powersave_anim(enabled))
    }
}

pub struct CtrlAnime {
    // node: HidRaw,
    node: Node,
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
    pub fn new(config: AnimeConfig) -> Result<CtrlAnime, RogError> {
        let usb = USBRaw::new(0x193b).ok();
        let hid = HidRaw::new("193b").ok();
        let node = if usb.is_some() {
            unsafe { Node::Usb(usb.unwrap_unchecked()) }
        } else if hid.is_some() {
            unsafe { Node::Hid(hid.unwrap_unchecked().0) }
        } else {
            return Err(RogError::Anime(AnimeError::NoDevice));
        };

        // TODO: something better to set wakeups disabled
        if matches!(node, Node::Usb(_)) {
            if let Ok(mut enumerator) = udev::Enumerator::new() {
                enumerator.match_subsystem("usb").ok();
                enumerator.match_attribute("idProduct", "193b").ok();

                if let Ok(mut enumer) = enumerator.scan_devices() {
                    if let Some(mut dev) = enumer.next() {
                        dev.set_attribute_value("power/wakeup", "disabled").ok();
                    }
                }
            }
        }

        let mut anime_type = get_anime_type()?;
        if let AnimeType::Unknown = anime_type {
            if let Some(model) = config.model_override {
                warn!("Overriding the Animatrix type as {model:?}");
                anime_type = model;
            }
        }

        info!("Device has an AniMe Matrix display: {anime_type:?}");
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

    /// Start an action thread. This is classed as a singleton and there should
    /// be only one running - so the thread uses atomics to signal run/exit.
    ///
    /// Because this also writes to the usb device, other write tries (display
    /// only) *must* get the mutex lock and set the `thread_exit` atomic.
    async fn run_thread(inner: Arc<Mutex<CtrlAnime>>, actions: Vec<ActionData>, mut once: bool) {
        if actions.is_empty() {
            warn!("AniMe system actions was empty");
            return;
        }

        if let Some(lock) = inner.try_lock() {
            lock.node
                .write_bytes(&pkt_set_enable_powersave_anim(false))
                .map_err(|err| {
                    warn!("rog_anime::run_animation:callback {}", err);
                })
                .ok();
        }

        // Loop rules:
        // - Lock the mutex **only when required**. That is, the lock must be held for
        //   the shortest duration possible.
        // - An AtomicBool used for thread exit should be checked in every loop,
        //   including nested

        // The only reason for this outer thread is to prevent blocking while waiting
        // for the next spawned thread to exit
        // TODO: turn this in to async task (maybe? COuld still risk blocking main
        // thread)
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
                    if let Some(lock) = inner.try_lock() {
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
                thread_running.store(true, Ordering::SeqCst);
                'main: loop {
                    for action in &actions {
                        if thread_exit.load(Ordering::SeqCst) {
                            break 'main;
                        }
                        match action {
                            ActionData::Animation(frames) => {
                                rog_anime::run_animation(frames, &|frame| {
                                    if thread_exit.load(Ordering::Acquire) {
                                        info!("rog-anime: animation sub-loop was asked to exit");
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
                                        .map_or_else(
                                            || {
                                                warn!("rog_anime::run_animation:callback failed");
                                                Err(AnimeError::NoFrames)
                                            },
                                            Ok,
                                        )
                                });
                                if thread_exit.load(Ordering::Acquire) {
                                    info!("rog-anime: sub-loop exited and main loop exiting now");
                                    break 'main;
                                }
                            }
                            ActionData::Image(image) => {
                                once = false;
                                if let Some(lock) = inner.try_lock() {
                                    lock.write_data_buffer(image.as_ref().clone())
                                        .map_err(|e| error!("{}", e))
                                        .ok();
                                }
                            }
                            ActionData::Pause(duration) => sleep(*duration),
                            ActionData::AudioEq
                            | ActionData::SystemInfo
                            | ActionData::TimeDate
                            | ActionData::Matrix => {}
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
                if let Some(lock) = inner.try_lock() {
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
                    lock.node
                        .write_bytes(&pkt_set_enable_powersave_anim(
                            lock.config.builtin_anims_enabled,
                        ))
                        .map_err(|err| {
                            warn!("rog_anime::run_animation:callback {}", err);
                        })
                        .ok();
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
            let mut bright = *led as f32;
            if bright > 254.0 {
                bright = 254.0;
            }
            *led = bright as u8;
        }
        let data = AnimePacketType::try_from(buffer)?;
        for row in &data {
            self.node.write_bytes(row)?;
        }
        self.node.write_bytes(&pkt_flush())?;
        Ok(())
    }

    fn do_initialization(&self) -> Result<(), RogError> {
        let pkts = pkts_for_init();
        self.node.write_bytes(&pkts[0])?;
        self.node.write_bytes(&pkts[1])?;
        Ok(())
    }
}

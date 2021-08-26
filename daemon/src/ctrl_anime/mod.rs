pub mod config;
pub mod zbus;

use ::zbus::Connection;
use log::{error, info, warn};
use logind_zbus::ManagerProxy;
use rog_anime::{
    usb::{
        pkt_for_apply, pkt_for_flush, pkt_for_set_boot, pkt_for_set_on, pkts_for_init, PROD_ID,
        VENDOR_ID,
    },
    ActionData, AnimeDataBuffer, AnimePacketType, ANIME_DATA_LEN,
};
use rog_supported::AnimeSupportedFunctions;
use rusb::{Device, DeviceHandle};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread::sleep,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::{error::RogError, GetSupported};

use self::config::{AnimeConfig, AnimeConfigCached};

impl GetSupported for CtrlAnime {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(CtrlAnime::get_device(VENDOR_ID, PROD_ID).is_ok())
    }
}

pub struct CtrlAnime {
    handle: DeviceHandle<rusb::GlobalContext>,
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
        // We don't expect this ID to ever change
        let device = CtrlAnime::get_device(0x0b05, 0x193b)?;

        let mut device = device.open()?;
        device.reset()?;

        device.set_auto_detach_kernel_driver(true).map_err(|err| {
            error!("Auto-detach kernel driver failed: {}", err);
            err
        })?;

        device.claim_interface(0).map_err(|err| {
            error!("Could not claim device interface: {}", err);
            err
        })?;

        info!("Device has an AniMe Matrix display");
        let mut cache = AnimeConfigCached::default();
        cache.init_from_config(&config)?;

        let ctrl = CtrlAnime {
            handle: device,
            cache,
            config,
            thread_exit: Arc::new(AtomicBool::new(false)),
            thread_running: Arc::new(AtomicBool::new(false)),
        };
        ctrl.do_initialization();

        Ok(ctrl)
    }

    fn get_device(vendor: u16, product: u16) -> Result<Device<rusb::GlobalContext>, rusb::Error> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return Ok(device);
            }
        }
        Err(rusb::Error::NoDevice)
    }

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
                info!("AniMe system thread started");
                // Getting copies of these Atomics is done *in* the thread to ensure
                // we don't block other threads/main
                let thread_exit;
                let thread_running;
                // First two loops are to ensure we *do* aquire a lock on the mutex
                // The reason the loop is required is because the USB writes can block
                // for up to 10ms. We can't fail to get the atomics.
                loop {
                    if let Ok(lock) = inner.try_lock() {
                        thread_exit = lock.thread_exit.clone();
                        thread_running = lock.thread_running.clone();
                        // Make any running loop exit first
                        thread_exit.store(true, Ordering::SeqCst);
                        break;
                    }
                }

                loop {
                    // wait for other threads to set not running so we know they exited
                    if !thread_running.load(Ordering::SeqCst) {
                        thread_exit.store(false, Ordering::SeqCst);
                        info!("AniMe forced a thread to exit");
                        break;
                    }
                }

                'main: loop {
                    if thread_exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
                    for action in actions.iter() {
                        match action {
                            ActionData::Animation(frames) => {
                                rog_anime::run_animation(frames, thread_exit.clone(), &|frame| {
                                    if let Ok(lock) = inner.try_lock() {
                                        lock.write_data_buffer(frame);
                                    }
                                })
                                .unwrap();

                                if thread_exit.load(Ordering::SeqCst) {
                                    break 'main;
                                }
                            }
                            ActionData::Image(image) => {
                                once = false;
                                if let Ok(lock) = inner.try_lock() {
                                    lock.write_data_buffer(image.as_ref().clone())
                                }
                            }
                            ActionData::Pause(duration) => sleep(*duration),
                            ActionData::AudioEq => {}
                            ActionData::SystemInfo => {}
                            ActionData::TimeDate => {}
                            ActionData::Matrix => {}
                        }
                    }
                    if once || actions.is_empty() {
                        break 'main;
                    }
                }
                // Clear the display on exit
                if let Ok(lock) = inner.try_lock() {
                    let data = AnimeDataBuffer::from_vec([0u8; ANIME_DATA_LEN].to_vec());
                    lock.write_data_buffer(data);
                }
                // Loop ended, set the atmonics
                thread_exit.store(false, Ordering::SeqCst);
                thread_running.store(false, Ordering::SeqCst);
                info!("AniMe system thread exited");
            })
            .map(|err| info!("AniMe system thread: {:?}", err))
            .ok();
    }

    fn write_bytes(&self, message: &[u8]) {
        match self.handle.write_control(
            0x21,  // request_type
            0x09,  // request
            0x35e, // value
            0x00,  // index
            message,
            Duration::from_millis(200),
        ) {
            Ok(_) => {}
            Err(err) => match err {
                rusb::Error::Timeout => {}
                _ => error!("Failed to write to led interrupt: {}", err),
            },
        }
    }

    /// Write only a data packet. This will modify the leds brightness using the
    /// global brightness set in config.
    fn write_data_buffer(&self, mut buffer: AnimeDataBuffer) {
        for led in buffer.get_mut()[7..].iter_mut() {
            let mut bright = *led as f32 * self.config.brightness;
            if bright > 254.0 {
                bright = 254.0;
            }
            *led = bright as u8;
        }
        let data = AnimePacketType::from(buffer);
        for row in data.iter() {
            self.write_bytes(row);
        }
        self.write_bytes(&pkt_for_flush());
    }

    fn do_initialization(&self) {
        let pkts = pkts_for_init();
        self.write_bytes(&pkts[0]);
        self.write_bytes(&pkts[1]);
    }
}

pub struct CtrlAnimeTask<'a> {
    inner: Arc<Mutex<CtrlAnime>>,
    _c: Connection,
    manager: ManagerProxy<'a>,
}

impl<'a> CtrlAnimeTask<'a> {
    pub fn new(inner: Arc<Mutex<CtrlAnime>>) -> Self {
        let connection = Connection::new_system().unwrap();

        let manager = ManagerProxy::new(&connection).unwrap();

        let c1 = inner.clone();
        // Run this action when the system starts shutting down
        manager
            .connect_prepare_for_shutdown(move |shutdown| {
                if shutdown {
                    'outer: loop {
                        if let Ok(lock) = c1.try_lock() {
                            lock.thread_exit.store(true, Ordering::SeqCst);
                            CtrlAnime::run_thread(c1.clone(), lock.cache.shutdown.clone(), false);
                            break 'outer;
                        }
                    }
                }
                Ok(())
            })
            .map_err(|err| {
                warn!("CtrlAnimeTask: new() {}", err);
                err
            })
            .ok();

        let c1 = inner.clone();
        // Run this action when the system wakes up from sleep
        manager
            .connect_prepare_for_sleep(move |sleep| {
                if !sleep {
                    // wait a fraction for things to wake up properly
                    std::thread::sleep(Duration::from_millis(100));
                    'outer: loop {
                        if let Ok(lock) = c1.try_lock() {
                            lock.thread_exit.store(true, Ordering::SeqCst);
                            CtrlAnime::run_thread(c1.clone(), lock.cache.wake.clone(), true);
                            break 'outer;
                        }
                    }
                }
                Ok(())
            })
            .map_err(|err| {
                warn!("CtrlAnimeTask: new() {}", err);
                err
            })
            .ok();

        Self {
            inner,
            _c: connection,
            manager,
        }
    }
}

impl<'a> crate::CtrlTask for CtrlAnimeTask<'a> {
    fn do_task(&self) -> Result<(), RogError> {
        if let Ok(mut lock) = self.inner.try_lock() {
            // Refresh the config and cache incase the user has edited it
            let config = AnimeConfig::load();
            lock.cache
                .init_from_config(&config)
                .map_err(|err| {
                    warn!("CtrlAnimeTask: do_task {}", err);
                    err
                })
                .ok();
        }

        // Check for signals on each task iteration, this will run the callbacks
        // if any signal is recieved
        self.manager.next_signal()?;
        Ok(())
    }
}

pub struct CtrlAnimeReloader(pub Arc<Mutex<CtrlAnime>>);

impl crate::Reloadable for CtrlAnimeReloader {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(lock) = self.0.try_lock() {
            lock.write_bytes(&pkt_for_set_on(lock.config.awake_enabled));
            lock.write_bytes(&pkt_for_apply());
            lock.write_bytes(&pkt_for_set_boot(lock.config.boot_anim_enabled));
            lock.write_bytes(&pkt_for_apply());

            let action = lock.cache.boot.clone();
            CtrlAnime::run_thread(self.0.clone(), action, true);
        }
        Ok(())
    }
}

use log::{error, info, warn};
use rog_anime::{
    usb::{
        pkt_for_apply, pkt_for_flush, pkt_for_set_boot, pkt_for_set_on, pkts_for_init, PROD_ID,
        VENDOR_ID,
    },
    ActionData, AnimTime, AnimeDataBuffer, AnimePacketType, ANIME_DATA_LEN,
};
use rog_types::supported::AnimeSupportedFunctions;
use rusb::{Device, DeviceHandle};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Instant,
};
use std::{sync::atomic::AtomicBool, time::Duration};
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::{
    config_anime::{AnimeConfig, AnimeConfigCached},
    error::RogError,
    GetSupported,
};

impl GetSupported for CtrlAnime {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(CtrlAnime::get_device(VENDOR_ID, PROD_ID).is_ok())
    }
}

pub struct CtrlAnime {
    handle: DeviceHandle<rusb::GlobalContext>,
    cache: AnimeConfigCached,
    _config: AnimeConfig,
    // set to force thread to exit
    thread_exit: AtomicBool,
    // Set to false when the thread exits
    thread_running: AtomicBool,
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
            _config: config,
            thread_exit: AtomicBool::new(false),
            thread_running: AtomicBool::new(false),
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

    // DOUBLE THREAD NEST!
    fn run_thread(inner: Arc<Mutex<CtrlAnime>>, action: Option<ActionData>, mut once: bool) {
        std::thread::Builder::new()
            .name("AniMe system thread start".into())
            .spawn(move || {
                // Make the loop exit first
                loop {
                    if let Ok(lock) = inner.try_lock() {
                        lock.thread_exit
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                        break;
                    }
                }
                loop {
                    if let Ok(lock) = inner.try_lock() {
                        if !lock
                            .thread_running
                            .load(std::sync::atomic::Ordering::SeqCst)
                        {
                            lock.thread_exit
                                .store(false, std::sync::atomic::Ordering::SeqCst);
                            info!("AniMe system thread exited");
                            break;
                        }
                    }
                }

                std::thread::Builder::new()
                    .name("AniMe system actions".into())
                    .spawn(move || {
                        info!("AniMe system thread started");
                        'main: loop {
                            if let Ok(lock) = inner.try_lock() {
                                if !once
                                    && lock.thread_exit.load(std::sync::atomic::Ordering::SeqCst)
                                {
                                    break 'main;
                                }
                                if let Some(ref action) = action {
                                    match action {
                                        ActionData::Animation(frames) => {
                                            let mut count = 0;
                                            let start = Instant::now();
                                            'animation: loop {
                                                for frame in frames.frames() {
                                                    lock.write_data_buffer(frame.frame().clone());
                                                    if let AnimTime::Time(time) = frames.duration()
                                                    {
                                                        if Instant::now().duration_since(start)
                                                            > time
                                                        {
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
                                            once = false;
                                            lock.write_data_buffer(image.as_ref().clone())
                                        }
                                        ActionData::Pause(_) => {}
                                        ActionData::AudioEq => {}
                                        ActionData::SystemInfo => {}
                                        ActionData::TimeDate => {}
                                        ActionData::Matrix => {}
                                    }
                                } else {
                                    break 'main;
                                }
                                if once {
                                    let data =
                                        AnimeDataBuffer::from_vec([0u8; ANIME_DATA_LEN].to_vec());
                                    lock.write_data_buffer(data);
                                    break 'main;
                                }
                            }
                        }
                        'exit: loop {
                            if let Ok(lock) = inner.try_lock() {
                                lock.thread_exit
                                    .store(false, std::sync::atomic::Ordering::SeqCst);
                                lock.thread_running
                                    .store(false, std::sync::atomic::Ordering::SeqCst);
                                break 'exit;
                            }
                        }
                    })
                    .map(|err| info!("AniMe system thread: {:?}", err))
                    .ok();
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

    fn write_data_buffer(&self, buffer: AnimeDataBuffer) {
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

pub struct CtrlAnimeTask(pub Arc<Mutex<CtrlAnime>>);

impl crate::CtrlTask for CtrlAnimeTask {
    fn do_task(&self) -> Result<(), RogError> {
        Ok(())
    }
}

pub struct CtrlAnimeReloader(pub Arc<Mutex<CtrlAnime>>);

impl crate::Reloadable for CtrlAnimeReloader {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(lock) = self.0.try_lock() {
            let action = lock.cache.boot.clone();
            CtrlAnime::run_thread(self.0.clone(), action, true);
        }
        Ok(())
    }
}

pub struct CtrlAnimeZbus(pub Arc<Mutex<CtrlAnime>>);

/// The struct with the main dbus methods requires this trait
impl crate::ZbusAdd for CtrlAnimeZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Anime"),
                self,
            )
            .map_err(|err| {
                warn!("CtrlAnimeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

// None of these calls can be guarnateed to succeed unless we loop until okay
// If the try_lock *does* succeed then any other thread trying to lock will not grab it
// until we finish.
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnimeZbus {
    /// Writes a data stream of length. Will force system thread to exit until it is restarted
    fn write(&self, input: AnimeDataBuffer) {
        'outer: loop {
            if let Ok(lock) = self.0.try_lock() {
                lock.thread_exit
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                lock.write_data_buffer(input);
                break 'outer;
            }
        }
    }

    fn set_on_off(&self, status: bool) {
        'outer: loop {
            if let Ok(lock) = self.0.try_lock() {
                lock.write_bytes(&pkt_for_set_on(status));
                break 'outer;
            }
        }
    }

    fn set_boot_on_off(&self, on: bool) {
        'outer: loop {
            if let Ok(lock) = self.0.try_lock() {
                lock.write_bytes(&pkt_for_set_boot(on));
                lock.write_bytes(&pkt_for_apply());
                break 'outer;
            }
        }
    }

    fn run_main_loop(&self, on: bool) {
        'outer: loop {
            if let Ok(lock) = self.0.try_lock() {
                lock.thread_exit
                    .store(on, std::sync::atomic::Ordering::SeqCst);
                CtrlAnime::run_thread(self.0.clone(), lock.cache.system.clone(), false);
                break 'outer;
            }
        }
    }
}

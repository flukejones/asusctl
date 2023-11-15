use std::sync::atomic::Ordering;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::warn;
use rog_anime::usb::{
    pkt_set_brightness, pkt_set_builtin_animations, pkt_set_enable_display,
    pkt_set_enable_powersave_anim, AnimAwake, AnimBooting, AnimShutdown, AnimSleeping, Brightness,
};
use rog_anime::{AnimeDataBuffer, DeviceState};
use zbus::export::futures_util::lock::Mutex;
use zbus::{dbus_interface, Connection, SignalContext};

use super::CtrlAnime;
use crate::error::RogError;

pub(super) const ZBUS_PATH: &str = "/org/asuslinux/Anime";

#[derive(Clone)]
pub struct CtrlAnimeZbus(pub Arc<Mutex<CtrlAnime>>);

/// The struct with the main dbus methods requires this trait
#[async_trait]
impl crate::ZbusRun for CtrlAnimeZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, ZBUS_PATH, server).await;
    }
}

// None of these calls can be guarnateed to succeed unless we loop until okay
// If the try_lock *does* succeed then any other thread trying to lock will not
// grab it until we finish.
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnimeZbus {
    /// Writes a data stream of length. Will force system thread to exit until
    /// it is restarted
    async fn write(&self, input: AnimeDataBuffer) -> zbus::fdo::Result<()> {
        let lock = self.0.lock().await;
        lock.thread_exit.store(true, Ordering::SeqCst);
        lock.write_data_buffer(input).map_err(|err| {
            warn!("ctrl_anime::run_animation:callback {}", err);
            err
        })?;
        Ok(())
    }

    /// Set base brightness level
    async fn set_brightness(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        brightness: Brightness,
    ) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_brightness(brightness))
            .map_err(|err| {
                warn!("ctrl_anime::set_brightness {}", err);
            })
            .ok();
        lock.node
            .write_bytes(&pkt_set_enable_display(brightness != Brightness::Off))
            .map_err(|err| {
                warn!("ctrl_anime::set_brightness {}", err);
            })
            .ok();

        lock.config.display_enabled = brightness != Brightness::Off;
        lock.config.display_brightness = brightness;
        lock.config.write();

        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Enable the builtin animations or not. This is quivalent to "Powersave
    /// animations" in Armory crate
    async fn set_builtins_enabled(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut lock = self.0.lock().await;
        lock.node
            .set_builtins_enabled(enabled, lock.config.display_brightness)
            .map_err(|err| {
                warn!("ctrl_anime::set_builtins_enabled {}", err);
            })
            .ok();

        if !enabled {
            let data = vec![255u8; lock.anime_type.data_length()];
            if let Ok(tmp) = AnimeDataBuffer::from_vec(lock.anime_type, data).map_err(|err| {
                warn!("ctrl_anime::set_builtins_enabled {}", err);
            }) {
                lock.node
                    .write_bytes(tmp.data())
                    .map_err(|err| {
                        warn!("ctrl_anime::set_builtins_enabled {}", err);
                    })
                    .ok();
            }
        }

        lock.config.display_enabled = enabled;
        lock.config.builtin_anims_enabled = enabled;
        lock.config.write();
        if enabled {
            lock.thread_exit.store(true, Ordering::Release);
        }

        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Set which builtin animation is used for each stage
    async fn set_builtin_animations(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        boot: AnimBooting,
        awake: AnimAwake,
        sleep: AnimSleeping,
        shutdown: AnimShutdown,
    ) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_builtin_animations(boot, awake, sleep, shutdown))
            .map_err(|err| {
                warn!("ctrl_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.node
            .write_bytes(&pkt_set_enable_powersave_anim(true))
            .map_err(|err| {
                warn!("ctrl_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.display_enabled = true;
        lock.config.builtin_anims.boot = boot;
        lock.config.builtin_anims.sleep = sleep;
        lock.config.builtin_anims.awake = awake;
        lock.config.builtin_anims.shutdown = shutdown;
        lock.config.write();

        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Set whether the AniMe is enabled at all
    async fn set_enable_display(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_enable_display(enabled))
            .map_err(|err| {
                warn!("ctrl_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.display_enabled = enabled;
        lock.config.write();

        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Set if to turn the AniMe Matrix off when external power is unplugged
    async fn set_off_when_unplugged(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut lock = self.0.lock().await;
        lock.config.off_when_unplugged = enabled;
        lock.config.write();
        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Set if to turn the AniMe Matrix off when the laptop is suspended
    async fn set_off_when_suspended(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut lock = self.0.lock().await;
        lock.config.off_when_suspended = enabled;
        lock.config.write();
        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// Set if to turn the AniMe Matrix off when the lid is closed
    async fn set_off_when_lid_closed(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut lock = self.0.lock().await;
        lock.config.off_when_lid_closed = enabled;
        lock.config.write();
        Self::notify_device_state(&ctxt, DeviceState::from(&lock.config))
            .await
            .ok();
    }

    /// The main loop is the base system set action if the user isn't running
    /// the user daemon
    async fn run_main_loop(&self, start: bool) {
        if start {
            let lock = self.0.lock().await;
            lock.thread_exit.store(true, Ordering::SeqCst);
            CtrlAnime::run_thread(self.0.clone(), lock.cache.system.clone(), false).await;
        }
    }

    /// Get the device state as stored by asusd
    // #[dbus_interface(property)]
    async fn device_state(&self) -> DeviceState {
        let lock = self.0.lock().await;
        DeviceState {
            display_enabled: lock.config.display_enabled,
            display_brightness: lock.config.display_brightness,
            builtin_anims_enabled: lock.config.builtin_anims_enabled,
            builtin_anims: lock.config.builtin_anims,
            off_when_unplugged: lock.config.off_when_unplugged,
            off_when_suspended: lock.config.off_when_suspended,
            off_when_lid_closed: lock.config.off_when_lid_closed,
        }
    }

    /// Notify listeners of the status of AniMe LED power and factory
    /// system-status animations
    #[dbus_interface(signal)]
    async fn notify_device_state(ctxt: &SignalContext<'_>, data: DeviceState) -> zbus::Result<()>;
}

#[async_trait]
impl crate::CtrlTask for CtrlAnimeZbus {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, _: SignalContext<'static>) -> Result<(), RogError> {
        let inner1 = self.0.clone();
        let inner2 = self.0.clone();
        let inner3 = self.0.clone();
        let inner4 = self.0.clone();
        self.create_sys_event_tasks(
            move |sleeping| {
                // on_sleep
                let inner = inner1.clone();
                async move {
                    let lock = inner.lock().await;
                    if lock.config.display_enabled {
                        if lock.config.off_when_suspended {
                            lock.node
                                .write_bytes(&pkt_set_enable_display(!sleeping))
                                .map_err(|err| {
                                    warn!("create_sys_event_tasks::off_when_suspended {}", err);
                                })
                                .ok();
                        }
                        if !lock.config.builtin_anims_enabled {
                            if sleeping {
                                CtrlAnime::run_thread(
                                    inner.clone(),
                                    lock.cache.sleep.clone(),
                                    true,
                                )
                                .await;
                            } else {
                                CtrlAnime::run_thread(inner.clone(), lock.cache.wake.clone(), true)
                                    .await;
                            }
                        }
                    }
                }
            },
            move |shutting_down| {
                // on_shutdown
                let inner = inner2.clone();
                async move {
                    let lock = inner.lock().await;
                    if lock.config.display_enabled && !lock.config.builtin_anims_enabled {
                        if shutting_down {
                            CtrlAnime::run_thread(inner.clone(), lock.cache.shutdown.clone(), true)
                                .await;
                        } else {
                            CtrlAnime::run_thread(inner.clone(), lock.cache.boot.clone(), true)
                                .await;
                        }
                    }
                }
            },
            move |lid_closed| {
                let inner = inner3.clone();
                // on lid change
                async move {
                    let lock = inner.lock().await;
                    if lock.config.off_when_lid_closed {
                        lock.node
                            .write_bytes(&pkt_set_enable_display(lid_closed))
                            .map_err(|err| {
                                warn!("create_sys_event_tasks::off_when_lid_closed {}", err);
                            })
                            .ok();
                    }
                }
            },
            move |power_plugged| {
                let inner = inner4.clone();
                // on power change
                async move {
                    let lock = inner.lock().await;
                    if lock.config.off_when_unplugged {
                        lock.node
                            .write_bytes(&pkt_set_enable_display(power_plugged))
                            .map_err(|err| {
                                warn!("create_sys_event_tasks::off_when_unplugged {}", err);
                            })
                            .ok();
                    }
                }
            },
        )
        .await;

        Ok(())
    }
}

#[async_trait]
impl crate::Reloadable for CtrlAnimeZbus {
    async fn reload(&mut self) -> Result<(), RogError> {
        if let Some(lock) = self.0.try_lock() {
            let anim = &lock.config.builtin_anims;
            // Set builtins
            if lock.config.builtin_anims_enabled {
                lock.node.write_bytes(&pkt_set_builtin_animations(
                    anim.boot,
                    anim.awake,
                    anim.sleep,
                    anim.shutdown,
                ))?;
            }
            // Builtins enabled or na?
            lock.node.set_builtins_enabled(
                lock.config.builtin_anims_enabled,
                lock.config.display_brightness,
            )?;

            if !lock.config.builtin_anims_enabled && !lock.cache.boot.is_empty() {
                lock.node
                    .write_bytes(&pkt_set_enable_powersave_anim(false))
                    .ok();

                let action = lock.cache.boot.clone();
                CtrlAnime::run_thread(self.0.clone(), action, true).await;
            }
        }
        Ok(())
    }
}

use std::sync::atomic::Ordering;
use std::sync::Arc;

use config_traits::StdConfig;
use log::warn;
use logind_zbus::manager::ManagerProxy;
use rog_anime::usb::{
    pkt_set_brightness, pkt_set_builtin_animations, pkt_set_enable_display,
    pkt_set_enable_powersave_anim, Brightness,
};
use rog_anime::{Animations, AnimeDataBuffer, DeviceState};
use zbus::export::futures_util::lock::Mutex;
use zbus::{interface, CacheProperties, Connection, SignalContext};

use super::CtrlAnime;
use crate::error::RogError;

pub const ANIME_ZBUS_NAME: &str = "Anime";
pub const ANIME_ZBUS_PATH: &str = "/org/asuslinux";

async fn get_logind_manager<'a>() -> ManagerProxy<'a> {
    let connection = Connection::system()
        .await
        .expect("Controller could not create dbus connection");

    ManagerProxy::builder(&connection)
        .cache_properties(CacheProperties::No)
        .build()
        .await
        .expect("Controller could not create ManagerProxy")
}

#[derive(Clone)]
pub struct CtrlAnimeZbus(pub Arc<Mutex<CtrlAnime>>);

/// The struct with the main dbus methods requires this trait
impl crate::ZbusRun for CtrlAnimeZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, ANIME_ZBUS_PATH, server).await;
    }
}

// None of these calls can be guarnateed to succeed unless we loop until okay
// If the try_lock *does* succeed then any other thread trying to lock will not
// grab it until we finish.
#[interface(name = "org.asuslinux.Anime")]
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
    #[zbus(property)]
    async fn brightness(&self) -> Brightness {
        let lock = self.0.lock().await;
        lock.config.display_brightness
    }

    /// Set base brightness level
    #[zbus(property)]
    async fn set_brightness(&self, brightness: Brightness) {
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
    }

    #[zbus(property)]
    async fn builtins_enabled(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.builtin_anims_enabled
    }

    /// Enable the builtin animations or not. This is quivalent to "Powersave
    /// animations" in Armory crate
    #[zbus(property)]
    async fn set_builtins_enabled(&self, enabled: bool) {
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

        lock.config.builtin_anims_enabled = enabled;
        lock.config.write();
        if enabled {
            lock.thread_exit.store(true, Ordering::Release);
        }
    }

    #[zbus(property)]
    async fn builtin_animations(&self) -> Animations {
        let lock = self.0.lock().await;
        lock.config.builtin_anims
    }

    /// Set which builtin animation is used for each stage
    #[zbus(property)]
    async fn set_builtin_animations(&self, settings: Animations) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_builtin_animations(
                settings.boot,
                settings.awake,
                settings.sleep,
                settings.shutdown,
            ))
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
        lock.config.builtin_anims = settings;
        lock.config.write();
    }

    #[zbus(property)]
    async fn enable_display(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.display_enabled
    }

    /// Set whether the AniMe is enabled at all
    #[zbus(property)]
    async fn set_enable_display(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_enable_display(enabled))
            .map_err(|err| {
                warn!("ctrl_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.display_enabled = enabled;
        lock.config.write();
    }

    #[zbus(property)]
    async fn off_when_unplugged(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.off_when_unplugged
    }

    /// Set if to turn the AniMe Matrix off when external power is unplugged
    #[zbus(property)]
    async fn set_off_when_unplugged(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        let manager = get_logind_manager().await;
        let pow = manager.on_external_power().await.unwrap_or_default();

        lock.node
            .write_bytes(&pkt_set_enable_display(!pow && !enabled))
            .map_err(|err| {
                warn!("create_sys_event_tasks::off_when_lid_closed {}", err);
            })
            .ok();

        lock.config.off_when_unplugged = enabled;
        lock.config.write();
    }

    #[zbus(property)]
    async fn off_when_suspended(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.off_when_suspended
    }

    /// Set if to turn the AniMe Matrix off when the laptop is suspended
    #[zbus(property)]
    async fn set_off_when_suspended(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        lock.config.off_when_suspended = enabled;
        lock.config.write();
    }

    #[zbus(property)]
    async fn off_when_lid_closed(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.off_when_lid_closed
    }

    /// Set if to turn the AniMe Matrix off when the lid is closed
    #[zbus(property)]
    async fn set_off_when_lid_closed(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        let manager = get_logind_manager().await;
        let lid = manager.lid_closed().await.unwrap_or_default();

        lock.node
            .write_bytes(&pkt_set_enable_display(lid && !enabled))
            .map_err(|err| {
                warn!("create_sys_event_tasks::off_when_lid_closed {}", err);
            })
            .ok();

        lock.config.off_when_lid_closed = enabled;
        lock.config.write();
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
    // #[zbus(property)]
    async fn device_state(&self) -> DeviceState {
        let lock = self.0.lock().await;
        DeviceState::from(&lock.config)
    }
}

impl crate::CtrlTask for CtrlAnimeZbus {
    fn zbus_path() -> &'static str {
        ANIME_ZBUS_PATH
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
                        lock.thread_exit.store(true, Ordering::Release); // ensure clean slate

                        lock.node
                            .write_bytes(&pkt_set_enable_display(
                                !(sleeping && lock.config.off_when_suspended),
                            ))
                            .map_err(|err| {
                                warn!("create_sys_event_tasks::off_when_suspended {}", err);
                            })
                            .ok();

                        if lock.config.builtin_anims_enabled {
                            lock.node
                                .write_bytes(&pkt_set_enable_powersave_anim(
                                    !(sleeping && lock.config.off_when_suspended),
                                ))
                                .map_err(|err| {
                                    warn!("create_sys_event_tasks::off_when_suspended {}", err);
                                })
                                .ok();
                        } else if !sleeping && !lock.config.builtin_anims_enabled {
                            // Run custom wake animation
                            lock.node
                                .write_bytes(&pkt_set_enable_powersave_anim(false))
                                .ok(); // ensure builtins are disabled

                            CtrlAnime::run_thread(inner.clone(), lock.cache.wake.clone(), true)
                                .await;
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
                        if lock.config.builtin_anims_enabled {
                            lock.node
                                .write_bytes(&pkt_set_enable_powersave_anim(!lid_closed))
                                .map_err(|err| {
                                    warn!("create_sys_event_tasks::off_when_suspended {}", err);
                                })
                                .ok();
                        }
                        lock.node
                            .write_bytes(&pkt_set_enable_display(!lid_closed))
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
                        if lock.config.builtin_anims_enabled {
                            lock.node
                                .write_bytes(&pkt_set_enable_powersave_anim(power_plugged))
                                .map_err(|err| {
                                    warn!("create_sys_event_tasks::off_when_suspended {}", err);
                                })
                                .ok();
                        }
                        lock.node
                            .write_bytes(&pkt_set_enable_display(power_plugged))
                            .map_err(|err| {
                                warn!("create_sys_event_tasks::off_when_unplugged {}", err);
                            })
                            .ok();
                    } else {
                        lock.node
                            .write_bytes(&pkt_set_brightness(lock.config.brightness_on_battery))
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

            let manager = get_logind_manager().await;
            let lid_closed = manager.lid_closed().await.unwrap_or_default();
            let power_plugged = manager.on_external_power().await.unwrap_or_default();

            let turn_off = (lid_closed && lock.config.off_when_lid_closed)
                || (!power_plugged && lock.config.off_when_unplugged);
            lock.node
                .write_bytes(&pkt_set_enable_display(!turn_off))
                .map_err(|err| {
                    warn!("create_sys_event_tasks::reload {}", err);
                })
                .ok();

            if turn_off || !lock.config.display_enabled {
                lock.node.write_bytes(&pkt_set_enable_display(false))?;
                // early return so we don't run animation thread
                return Ok(());
            }

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

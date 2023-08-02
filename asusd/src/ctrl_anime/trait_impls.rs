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
            warn!("rog_anime::run_animation:callback {}", err);
            err
        })?;
        Ok(())
    }

    /// Set the global AniMe brightness
    async fn set_image_brightness(&self, bright: f32) {
        let mut lock = self.0.lock().await;
        let mut bright = bright;
        if bright < 0.0 {
            bright = 0.0;
        } else if bright > 1.0 {
            bright = 1.0;
        }
        lock.config.brightness = bright;
        lock.config.write();
    }

    /// Set base brightness level
    // TODO: enum for brightness
    async fn set_brightness(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        brightness: Brightness,
    ) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_brightness(brightness))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.display_brightness = brightness;
        lock.config.write();

        Self::notify_device_state(
            &ctxt,
            DeviceState {
                display_enabled: lock.config.display_enabled,
                display_brightness: lock.config.display_brightness,
                builtin_anims_enabled: lock.config.builtin_anims_enabled,
                builtin_anims: lock.config.builtin_anims,
            },
        )
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
        dbg!("LOOOOOOOOOOCKEDDD");
        lock.node
            .write_bytes(&pkt_set_enable_powersave_anim(enabled))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.builtin_anims_enabled = enabled;
        lock.config.write();
        if enabled {
            lock.thread_exit.store(true, Ordering::Release);
        }

        Self::notify_device_state(
            &ctxt,
            DeviceState {
                display_enabled: lock.config.display_enabled,
                display_brightness: lock.config.display_brightness,
                builtin_anims_enabled: lock.config.builtin_anims_enabled,
                builtin_anims: lock.config.builtin_anims,
            },
        )
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
            .write_bytes(&pkt_set_enable_powersave_anim(true))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.node
            .write_bytes(&pkt_set_builtin_animations(boot, awake, sleep, shutdown))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.builtin_anims.boot = boot;
        lock.config.builtin_anims.sleep = sleep;
        lock.config.builtin_anims.awake = awake;
        lock.config.builtin_anims.shutdown = shutdown;
        lock.config.write();

        Self::notify_device_state(
            &ctxt,
            DeviceState {
                display_enabled: lock.config.display_enabled,
                display_brightness: lock.config.display_brightness,
                builtin_anims_enabled: lock.config.builtin_anims_enabled,
                builtin_anims: lock.config.builtin_anims,
            },
        )
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
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.display_enabled = enabled;
        lock.config.write();

        Self::notify_device_state(
            &ctxt,
            DeviceState {
                display_enabled: lock.config.display_enabled,
                display_brightness: lock.config.display_brightness,
                builtin_anims_enabled: lock.config.builtin_anims_enabled,
                builtin_anims: lock.config.builtin_anims,
            },
        )
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
            move || {
                // on_sleep
                let inner1 = inner1.clone();
                async move {
                    let lock = inner1.lock().await;
                    CtrlAnime::run_thread(inner1.clone(), lock.cache.sleep.clone(), true).await;
                }
            },
            move || {
                // on_wake
                let inner2 = inner2.clone();
                async move {
                    let lock = inner2.lock().await;
                    CtrlAnime::run_thread(inner2.clone(), lock.cache.wake.clone(), true).await;
                }
            },
            move || {
                // on_shutdown
                let inner3 = inner3.clone();
                async move {
                    let lock = inner3.lock().await;
                    CtrlAnime::run_thread(inner3.clone(), lock.cache.shutdown.clone(), true).await;
                }
            },
            move || {
                // on_boot
                let inner4 = inner4.clone();
                async move {
                    let lock = inner4.lock().await;
                    CtrlAnime::run_thread(inner4.clone(), lock.cache.boot.clone(), true).await;
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
            lock.node
                .write_bytes(&pkt_set_enable_display(lock.config.display_enabled))?;
            lock.node.write_bytes(&pkt_set_enable_powersave_anim(
                lock.config.builtin_anims_enabled,
            ))?;
            lock.node.write_bytes(&pkt_set_builtin_animations(
                anim.boot,
                anim.awake,
                anim.sleep,
                anim.shutdown,
            ))?;

            if lock.config.builtin_anims_enabled && !lock.cache.boot.is_empty() {
                lock.node
                    .write_bytes(&pkt_set_enable_powersave_anim(false))
                    .ok();
            }
            let action = lock.cache.boot.clone();
            CtrlAnime::run_thread(self.0.clone(), action, true).await;
        }
        Ok(())
    }
}

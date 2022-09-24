use super::CtrlAnime;
use crate::error::RogError;
use async_trait::async_trait;
use log::{info, warn};
use rog_anime::{
    usb::{pkt_for_apply, pkt_for_set_boot, pkt_for_set_on},
    AnimeDataBuffer, AnimePowerStates,
};
use std::sync::{atomic::Ordering, Arc};
use zbus::{
    dbus_interface,
    export::futures_util::lock::{Mutex, MutexGuard},
    Connection, SignalContext,
};

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
// If the try_lock *does* succeed then any other thread trying to lock will not grab it
// until we finish.
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnimeZbus {
    /// Writes a data stream of length. Will force system thread to exit until it is restarted
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
    async fn set_brightness(&self, bright: f32) {
        let mut lock = self.0.lock().await;
        let mut bright = bright;
        if bright < 0.0 {
            bright = 0.0
        } else if bright > 1.0 {
            bright = 1.0;
        }
        lock.config.brightness = bright;
        lock.config.write();
    }

    /// Set whether the AniMe is displaying images/data
    async fn set_on_off(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>, status: bool) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_for_set_on(status))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.awake_enabled = status;
        lock.config.write();

        Self::notify_power_states(
            &ctxt,
            AnimePowerStates {
                brightness: lock.config.brightness.floor() as u8,
                enabled: lock.config.awake_enabled,
                boot_anim_enabled: lock.config.boot_anim_enabled,
            },
        )
        .await
        .ok();
    }

    /// Set whether the AniMe will show boot, suspend, or off animations
    async fn set_boot_on_off(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>, on: bool) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_for_set_boot(on))
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.node
            .write_bytes(&pkt_for_apply())
            .map_err(|err| {
                warn!("rog_anime::run_animation:callback {}", err);
            })
            .ok();
        lock.config.boot_anim_enabled = on;
        lock.config.write();

        Self::notify_power_states(
            &ctxt,
            AnimePowerStates {
                brightness: lock.config.brightness.floor() as u8,
                enabled: lock.config.awake_enabled,
                boot_anim_enabled: lock.config.boot_anim_enabled,
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
            CtrlAnime::run_thread(self.0.clone(), lock.cache.system.clone(), false);
        }
    }

    /// Get status of if the AniMe LEDs are on/displaying while system is awake
    #[dbus_interface(property)]
    async fn awake_enabled(&self) -> bool {
        let lock = self.0.lock().await;
        return lock.config.awake_enabled;
    }

    /// Get the status of if factory system-status animations are enabled
    #[dbus_interface(property)]
    async fn boot_enabled(&self) -> bool {
        let lock = self.0.lock().await;
        return lock.config.boot_anim_enabled;
    }

    /// Notify listeners of the status of AniMe LED power and factory system-status animations
    #[dbus_interface(signal)]
    async fn notify_power_states(
        ctxt: &SignalContext<'_>,
        data: AnimePowerStates,
    ) -> zbus::Result<()>;
}

#[async_trait]
impl crate::CtrlTask for CtrlAnimeZbus {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, _: SignalContext<'static>) -> Result<(), RogError> {
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

        let inner1 = self.0.clone();
        let inner2 = self.0.clone();
        let inner3 = self.0.clone();
        let inner4 = self.0.clone();
        self.create_sys_event_tasks(
            // Loop is required to try an attempt to get the mutex *without* blocking
            // other threads - it is possible to end up with deadlocks otherwise.
            move || loop {
                if let Some(lock) = inner1.try_lock() {
                    run_action(true, lock, inner1.clone());
                    break;
                }
            },
            move || loop {
                if let Some(lock) = inner2.try_lock() {
                    run_action(false, lock, inner2.clone());
                    break;
                }
            },
            move || loop {
                if let Some(lock) = inner3.try_lock() {
                    run_action(true, lock, inner3.clone());
                    break;
                }
            },
            move || loop {
                if let Some(lock) = inner4.try_lock() {
                    run_action(false, lock, inner4.clone());
                    break;
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

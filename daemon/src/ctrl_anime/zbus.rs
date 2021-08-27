use std::sync::{Arc, Mutex};

use log::warn;
use rog_anime::{
    usb::{pkt_for_apply, pkt_for_set_boot, pkt_for_set_on},
    AnimeDataBuffer, AnimePowerStates,
};
use zbus::dbus_interface;
use zvariant::ObjectPath;

use std::sync::atomic::Ordering;

use super::CtrlAnime;

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
                lock.thread_exit.store(true, Ordering::SeqCst);
                lock.write_data_buffer(input);
                break 'outer;
            }
        }
    }

    /// Set the global AniMe brightness
    fn set_brightness(&self, bright: f32) {
        'outer: loop {
            if let Ok(mut lock) = self.0.try_lock() {
                let mut bright = bright;
                if bright < 0.0 {
                    bright = 0.0
                } else if bright > 254.0 {
                    bright = 254.0;
                }
                lock.config.brightness = bright;
                lock.config.write();
                break 'outer;
            }
        }
    }

    /// Set whether the AniMe is displaying images/data
    fn set_on_off(&self, status: bool) {
        'outer: loop {
            if let Ok(mut lock) = self.0.try_lock() {
                lock.write_bytes(&pkt_for_set_on(status));
                lock.config.awake_enabled = status;
                lock.config.write();

                let states = AnimePowerStates {
                    enabled: lock.config.awake_enabled,
                    boot_anim_enabled: lock.config.boot_anim_enabled,
                };
                self.notify_power_states(&states)
                    .unwrap_or_else(|err| warn!("{}", err));
                break 'outer;
            }
        }
    }

    /// Set whether the AniMe will show boot, suspend, or off animations
    fn set_boot_on_off(&self, on: bool) {
        'outer: loop {
            if let Ok(mut lock) = self.0.try_lock() {
                lock.write_bytes(&pkt_for_set_boot(on));
                lock.write_bytes(&pkt_for_apply());
                lock.config.boot_anim_enabled = on;
                lock.config.write();

                let states = AnimePowerStates {
                    enabled: lock.config.awake_enabled,
                    boot_anim_enabled: lock.config.boot_anim_enabled,
                };
                self.notify_power_states(&states)
                    .unwrap_or_else(|err| warn!("{}", err));
                break 'outer;
            }
        }
    }

    /// The main loop is the base system set action if the user isn't running
    /// the user daemon
    fn run_main_loop(&self, start: bool) {
        if start {
            'outer: loop {
                if let Ok(lock) = self.0.try_lock() {
                    lock.thread_exit.store(true, Ordering::SeqCst);
                    CtrlAnime::run_thread(self.0.clone(), lock.cache.system.clone(), false);
                    break 'outer;
                }
            }
        }
    }

    /// Get status of if the AniMe LEDs are on
    #[dbus_interface(property)]
    fn awake_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.awake_enabled;
        }
        true
    }

    /// Get the status of if factory system-status animations are enabled
    #[dbus_interface(property)]
    fn boot_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.boot_anim_enabled;
        }
        true
    }

    /// Notify listeners of the status of AniMe LED power and factory system-status animations
    #[dbus_interface(signal)]
    fn notify_power_states(&self, data: &AnimePowerStates) -> zbus::Result<()>;
}

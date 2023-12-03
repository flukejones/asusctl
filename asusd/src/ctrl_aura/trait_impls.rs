use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::{debug, error, info, warn};
use rog_aura::advanced::UsbPackets;
use rog_aura::usb::{AuraDevice, AuraPowerDev};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, LedBrightness};
use zbus::export::futures_util::lock::{Mutex, MutexGuard};
use zbus::export::futures_util::StreamExt;
use zbus::fdo::Error as ZbErr;
use zbus::{dbus_interface, Connection, SignalContext};

use super::controller::CtrlKbdLed;
use crate::error::RogError;
use crate::CtrlTask;

pub const AURA_ZBUS_NAME: &str = "Aura";
pub const AURA_ZBUS_PATH: &str = "/org/asuslinux/Aura";

#[derive(Clone)]
pub struct CtrlAuraZbus(pub Arc<Mutex<CtrlKbdLed>>);

impl CtrlAuraZbus {
    fn update_config(lock: &mut CtrlKbdLed) -> Result<(), RogError> {
        let bright = lock.sysfs_node.get_brightness()?;
        lock.config.read();
        lock.config.brightness = bright.into();
        lock.config.write();
        Ok(())
    }
}

#[async_trait]
impl crate::ZbusRun for CtrlAuraZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, AURA_ZBUS_PATH, server).await;
    }
}

/// The main interface for changing, reading, or notfying signals
///
/// LED commands are split between Brightness, Modes, Per-Key
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAuraZbus {
    /// Return the device type for this Aura keyboard
    #[dbus_interface(property)]
    async fn device_type(&self) -> AuraDevice {
        let ctrl = self.0.lock().await;
        ctrl.led_prod
    }

    /// Return the current LED brightness
    #[dbus_interface(property)]
    async fn brightness(&self) -> Result<LedBrightness, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.sysfs_node.get_brightness().map(|n| n.into())?)
    }

    /// Set the keyboard brightness level (0-3)
    #[dbus_interface(property)]
    async fn set_brightness(&mut self, brightness: LedBrightness) -> Result<(), ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.sysfs_node.set_brightness(brightness.into())?)
    }

    /// Total levels of brightness available
    #[dbus_interface(property)]
    async fn supported_brightness(&self) -> Vec<LedBrightness> {
        vec![
            LedBrightness::Off,
            LedBrightness::Low,
            LedBrightness::Med,
            LedBrightness::High,
        ]
    }

    /// The total available modes
    #[dbus_interface(property)]
    async fn supported_modes(&self) -> Result<Vec<AuraModeNum>, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.config.builtins.keys().cloned().collect())
    }

    /// The current mode data
    #[dbus_interface(property)]
    async fn led_mode(&self) -> Result<AuraModeNum, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.config.current_mode)
    }

    /// Set an Aura effect if the effect mode or zone is supported.
    ///
    /// On success the aura config file is read to refresh cached values, then
    /// the effect is stored and config written to disk.
    #[dbus_interface(property)]
    async fn set_led_mode(&mut self, num: AuraModeNum) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        ctrl.config.current_mode = num;
        ctrl.write_current_config_mode()?;
        if ctrl.config.brightness == LedBrightness::Off {
            ctrl.config.brightness = LedBrightness::Med;
        }
        ctrl.sysfs_node
            .set_brightness(ctrl.config.brightness.into())?;
        ctrl.config.write();
        Ok(())
    }

    /// The current mode data
    #[dbus_interface(property)]
    async fn led_mode_data(&self) -> Result<AuraEffect, ZbErr> {
        let ctrl = self.0.lock().await;
        let mode = ctrl.config.current_mode;
        match ctrl.config.builtins.get(&mode) {
            Some(effect) => Ok(effect.clone()),
            None => Err(ZbErr::Failed("Could not get the current effect".into())),
        }
    }

    /// Set an Aura effect if the effect mode or zone is supported.
    ///
    /// On success the aura config file is read to refresh cached values, then
    /// the effect is stored and config written to disk.
    #[dbus_interface(property)]
    async fn set_led_mode_data(&mut self, effect: AuraEffect) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        if !ctrl.supported_modes.basic_modes.contains(&effect.mode)
            || effect.zone != AuraZone::None
                && !ctrl.supported_modes.basic_zones.contains(&effect.zone)
        {
            return Err(ZbErr::NotSupported(format!(
                "The Aura effect is not supported: {effect:?}"
            )));
        }

        ctrl.write_mode(&effect)?;
        if ctrl.config.brightness == LedBrightness::Off {
            ctrl.config.brightness = LedBrightness::Med;
        }
        ctrl.sysfs_node
            .set_brightness(ctrl.config.brightness.into())?;
        ctrl.config.set_builtin(effect);
        ctrl.config.write();
        Ok(())
    }

    /// Get the data set for every mode available
    async fn all_mode_data(&self) -> BTreeMap<AuraModeNum, AuraEffect> {
        let ctrl = self.0.lock().await;
        ctrl.config.builtins.clone()
    }

    // As property doesn't work for AuraPowerDev (complexity of serialization?)
    #[dbus_interface(property)]
    async fn led_power(&self) -> AuraPowerDev {
        let ctrl = self.0.lock().await;
        AuraPowerDev::from(&ctrl.config.enabled)
    }

    /// Set a variety of states, input is array of enum.
    /// `enabled` sets if the sent array should be disabled or enabled
    ///
    /// For Modern ROG devices the "enabled" flag is ignored.
    #[dbus_interface(property)]
    async fn set_led_power(&mut self, options: (AuraPowerDev, bool)) -> Result<(), ZbErr> {
        let enabled = options.1;
        let options = options.0;
        let mut ctrl = self.0.lock().await;
        for p in options.tuf {
            ctrl.config.enabled.set_tuf(p, enabled);
        }
        for p in options.old_rog {
            ctrl.config.enabled.set_0x1866(p, enabled);
        }
        ctrl.config.enabled.set_0x19b6(options.rog);
        ctrl.config.write();
        Ok(ctrl.set_power_states().map_err(|e| {
            warn!("{}", e);
            e
        })?)
    }

    /// On machine that have some form of either per-key keyboard or per-zone
    /// this can be used to write custom effects over dbus. The input is a
    /// nested `Vec<Vec<8>>` where `Vec<u8>` is a raw USB packet
    async fn direct_addressing_raw(&self, data: UsbPackets) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        ctrl.write_effect_block(&data)?;
        Ok(())
    }
}

#[async_trait]
impl CtrlTask for CtrlAuraZbus {
    fn zbus_path() -> &'static str {
        AURA_ZBUS_PATH
    }

    async fn create_tasks(&self, _: SignalContext<'static>) -> Result<(), RogError> {
        let load_save = |start: bool, mut lock: MutexGuard<'_, CtrlKbdLed>| {
            // If waking up
            if !start {
                info!("CtrlKbdLedTask reloading brightness and modes");
                lock.sysfs_node
                    .set_brightness(lock.config.brightness.into())
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
                lock.write_current_config_mode()
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
            } else if start {
                Self::update_config(&mut lock)
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
            }
        };

        let inner1 = self.0.clone();
        let inner3 = self.0.clone();
        self.create_sys_event_tasks(
            move |sleeping| {
                let inner1 = inner1.clone();
                async move {
                    let lock = inner1.lock().await;
                    load_save(sleeping, lock);
                }
            },
            move |_shutting_down| {
                let inner3 = inner3.clone();
                async move {
                    let lock = inner3.lock().await;
                    load_save(false, lock);
                }
            },
            move |_lid_closed| {
                // on lid change
                async move {}
            },
            move |_power_plugged| {
                // power change
                async move {}
            },
        )
        .await;

        let ctrl2 = self.0.clone();
        let ctrl = self.0.lock().await;
        let watch = ctrl.sysfs_node.monitor_brightness()?;
        tokio::spawn(async move {
            let mut buffer = [0; 32];
            watch
                .into_event_stream(&mut buffer)
                .unwrap()
                .for_each(|_| async {
                    if let Some(lock) = ctrl2.try_lock() {
                        load_save(true, lock);
                    }
                })
                .await;
        });

        Ok(())
    }
}

#[async_trait]
impl crate::Reloadable for CtrlAuraZbus {
    async fn reload(&mut self) -> Result<(), RogError> {
        let mut ctrl = self.0.lock().await;
        debug!("CtrlKbdLedZbus: reloading keyboard mode");
        ctrl.write_current_config_mode()?;
        debug!("CtrlKbdLedZbus: reloading power states");
        ctrl.set_power_states().map_err(|err| warn!("{err}")).ok();
        Ok(())
    }
}

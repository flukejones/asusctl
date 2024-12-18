use std::collections::BTreeMap;
use std::sync::Arc;

use config_traits::StdConfig;
use log::{debug, error, info, warn};
use rog_aura::keyboard::{LaptopAuraPower, UsbPackets};
use rog_aura::{AuraDeviceType, AuraEffect, AuraModeNum, AuraZone, LedBrightness, PowerZones};
use zbus::export::futures_util::lock::{Mutex, MutexGuard};
use zbus::export::futures_util::StreamExt;
use zbus::fdo::Error as ZbErr;
use zbus::interface;
use zbus::object_server::SignalEmitter;

use super::controller::CtrlKbdLed;
use crate::error::RogError;
use crate::CtrlTask;

pub const AURA_ZBUS_NAME: &str = "Aura";
pub const AURA_ZBUS_PATH: &str = "/org/asuslinux";

#[derive(Clone)]
pub struct CtrlAuraZbus(Arc<Mutex<CtrlKbdLed>>, SignalEmitter<'static>);

impl CtrlAuraZbus {
    pub fn new(controller: CtrlKbdLed, signal: SignalEmitter<'static>) -> Self {
        Self(Arc::new(Mutex::new(controller)), signal)
    }

    fn update_config(lock: &mut CtrlKbdLed) -> Result<(), RogError> {
        let bright = lock.led_node.get_brightness().unwrap_or_default();
        lock.config.read();
        lock.config.brightness = bright.into();
        lock.config.write();
        Ok(())
    }
}

/// The main interface for changing, reading, or notfying
///
/// LED commands are split between Brightness, Modes, Per-Key
#[interface(name = "org.asuslinux.Aura")]
impl CtrlAuraZbus {
    /// Return the device type for this Aura keyboard
    #[zbus(property)]
    async fn device_type(&self) -> AuraDeviceType {
        let ctrl = self.0.lock().await;
        ctrl.led_type
    }

    /// Return the current LED brightness
    #[zbus(property)]
    async fn brightness(&self) -> Result<LedBrightness, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.led_node.get_brightness().map(|n| n.into())?)
    }

    /// Set the keyboard brightness level (0-3)
    #[zbus(property)]
    async fn set_brightness(&mut self, brightness: LedBrightness) -> Result<(), ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.led_node.set_brightness(brightness.into())?)
    }

    /// Total levels of brightness available
    #[zbus(property)]
    async fn supported_brightness(&self) -> Vec<LedBrightness> {
        vec![
            LedBrightness::Off,
            LedBrightness::Low,
            LedBrightness::Med,
            LedBrightness::High,
        ]
    }

    /// The total available modes
    #[zbus(property)]
    async fn supported_basic_modes(&self) -> Result<Vec<AuraModeNum>, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.config.builtins.keys().cloned().collect())
    }

    #[zbus(property)]
    async fn supported_basic_zones(&self) -> Result<Vec<AuraZone>, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.supported_data.basic_zones.clone())
    }

    #[zbus(property)]
    async fn supported_power_zones(&self) -> Result<Vec<PowerZones>, ZbErr> {
        let ctrl = self.0.lock().await;
        Ok(ctrl.supported_data.power_zones.clone())
    }

    /// The current mode data
    #[zbus(property)]
    async fn led_mode(&self) -> Result<AuraModeNum, ZbErr> {
        // entirely possible to deadlock here, so use try instead of lock()
        // let ctrl = self.0.lock().await;
        // Ok(ctrl.config.current_mode)
        if let Some(ctrl) = self.0.try_lock() {
            Ok(ctrl.config.current_mode)
        } else {
            Err(ZbErr::Failed("Aura control couldn't lock self".to_string()))
        }
    }

    /// Set an Aura effect if the effect mode or zone is supported.
    ///
    /// On success the aura config file is read to refresh cached values, then
    /// the effect is stored and config written to disk.
    #[zbus(property)]
    async fn set_led_mode(&mut self, num: AuraModeNum) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        ctrl.config.current_mode = num;
        ctrl.write_current_config_mode()?;
        if ctrl.config.brightness == LedBrightness::Off {
            ctrl.config.brightness = LedBrightness::Med;
        }
        if ctrl.led_node.has_brightness_control() {
            ctrl.led_node
                .set_brightness(ctrl.config.brightness.into())?;
        }
        ctrl.config.write();

        self.led_mode_changed(&self.1).await.ok();
        self.led_mode_data_changed(&self.1).await.ok();
        Ok(())
    }

    /// The current mode data
    #[zbus(property)]
    async fn led_mode_data(&self) -> Result<AuraEffect, ZbErr> {
        // entirely possible to deadlock here, so use try instead of lock()
        if let Some(ctrl) = self.0.try_lock() {
            let mode = ctrl.config.current_mode;
            match ctrl.config.builtins.get(&mode) {
                Some(effect) => Ok(effect.clone()),
                None => Err(ZbErr::Failed("Could not get the current effect".into())),
            }
        } else {
            Err(ZbErr::Failed("Aura control couldn't lock self".to_string()))
        }
    }

    /// Set an Aura effect if the effect mode or zone is supported.
    ///
    /// On success the aura config file is read to refresh cached values, then
    /// the effect is stored and config written to disk.
    #[zbus(property)]
    async fn set_led_mode_data(&mut self, effect: AuraEffect) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        if !ctrl.supported_data.basic_modes.contains(&effect.mode)
            || effect.zone != AuraZone::None
                && !ctrl.supported_data.basic_zones.contains(&effect.zone)
        {
            return Err(ZbErr::NotSupported(format!(
                "The Aura effect is not supported: {effect:?}"
            )));
        }

        ctrl.write_effect_and_apply(&effect)?;
        if ctrl.config.brightness == LedBrightness::Off {
            ctrl.config.brightness = LedBrightness::Med;
        }
        if ctrl.led_node.has_brightness_control() {
            ctrl.led_node
                .set_brightness(ctrl.config.brightness.into())?;
        }
        ctrl.config.set_builtin(effect);
        ctrl.config.write();

        self.led_mode_changed(&self.1).await.ok();
        Ok(())
    }

    /// Get the data set for every mode available
    async fn all_mode_data(&self) -> BTreeMap<AuraModeNum, AuraEffect> {
        let ctrl = self.0.lock().await;
        ctrl.config.builtins.clone()
    }

    // As property doesn't work for AuraPowerDev (complexity of serialization?)
    #[zbus(property)]
    async fn led_power(&self) -> LaptopAuraPower {
        let ctrl = self.0.lock().await;
        ctrl.config.enabled.clone()
    }

    /// Set a variety of states, input is array of enum.
    /// `enabled` sets if the sent array should be disabled or enabled
    ///
    /// For Modern ROG devices the "enabled" flag is ignored.
    #[zbus(property)]
    async fn set_led_power(&mut self, options: LaptopAuraPower) -> Result<(), ZbErr> {
        let mut ctrl = self.0.lock().await;
        for opt in options.states {
            let zone = opt.zone;
            for config in ctrl.config.enabled.states.iter_mut() {
                if config.zone == zone {
                    *config = opt;
                }
            }
        }
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

impl CtrlTask for CtrlAuraZbus {
    fn zbus_path() -> &'static str {
        "/org/asuslinux"
    }

    async fn create_tasks(&self, _: SignalEmitter<'static>) -> Result<(), RogError> {
        let load_save =
            |start: bool, mut lock: MutexGuard<'_, CtrlKbdLed>| -> Result<(), RogError> {
                // If waking up
                if !start {
                    info!("CtrlKbdLedTask reloading brightness and modes");
                    if lock.led_node.has_brightness_control() {
                        lock.led_node
                            .set_brightness(lock.config.brightness.into())
                            .map_err(|e| {
                                error!("CtrlKbdLedTask: {e}");
                                e
                            })?;
                    }
                    lock.write_current_config_mode().map_err(|e| {
                        error!("CtrlKbdLedTask: {e}");
                        e
                    })?;
                } else if start {
                    Self::update_config(&mut lock).map_err(|e| {
                        error!("CtrlKbdLedTask: {e}");
                        e
                    })?;
                }
                Ok(())
            };

        let inner1 = self.0.clone();
        let inner3 = self.0.clone();
        self.create_sys_event_tasks(
            move |sleeping| {
                let inner1 = inner1.clone();
                async move {
                    let lock = inner1.lock().await;
                    load_save(sleeping, lock).unwrap(); // unwrap as we want to
                                                        // bomb out of the task
                }
            },
            move |_shutting_down| {
                let inner3 = inner3.clone();
                async move {
                    let lock = inner3.lock().await;
                    load_save(false, lock).unwrap(); // unwrap as we want to
                                                     // bomb out of the task
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
        if ctrl.led_node.has_brightness_control() {
            let watch = ctrl.led_node.monitor_brightness()?;
            tokio::spawn(async move {
                let mut buffer = [0; 32];
                watch
                    .into_event_stream(&mut buffer)
                    .unwrap()
                    .for_each(|_| async {
                        if let Some(lock) = ctrl2.try_lock() {
                            load_save(true, lock).unwrap(); // unwrap as we want
                                                            // to
                                                            // bomb out of the
                                                            // task
                        }
                    })
                    .await;
            });
        }

        Ok(())
    }
}

impl crate::Reloadable for CtrlAuraZbus {
    async fn reload(&mut self) -> Result<(), RogError> {
        let mut ctrl = self.0.lock().await;
        ctrl.fix_ally_power()?;
        debug!("reloading keyboard mode");
        ctrl.write_current_config_mode()?;
        debug!("reloading power states");
        ctrl.set_power_states().map_err(|err| warn!("{err}")).ok();
        Ok(())
    }
}

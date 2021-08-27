use log::{error, warn};
use rog_aura::{AuraEffect, LedBrightness, LedPowerStates};
use zbus::dbus_interface;
use zvariant::ObjectPath;

use super::controller::CtrlKbdLedZbus;

impl crate::ZbusAdd for CtrlKbdLedZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&ObjectPath::from_str_unchecked("/org/asuslinux/Led"), self)
            .map_err(|err| {
                error!("DbusKbdLed: add_to_server {}", err);
            })
            .ok();
    }
}

/// The main interface for changing, reading, or notfying signals
///
/// LED commands are split between Brightness, Modes, Per-Key
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlKbdLedZbus {
    /// Set the keyboard brightness level (0-3)
    fn set_brightness(&mut self, brightness: LedBrightness) {
        if let Ok(ctrl) = self.0.try_lock() {
            ctrl.set_brightness(brightness)
                .map_err(|err| warn!("{}", err))
                .ok();
        }
    }

    /// Set the keyboard LED to enabled while the device is awake
    fn set_awake_enabled(&mut self, enabled: bool) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_states_enabled(enabled, ctrl.config.sleep_anim_enabled)
                .map_err(|err| warn!("{}", err))
                .ok();
            ctrl.config.awake_enabled = enabled;
            ctrl.config.write();

            let states = LedPowerStates {
                enabled: ctrl.config.awake_enabled,
                sleep_anim_enabled: ctrl.config.sleep_anim_enabled,
            };
            self.notify_power_states(&states)
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    /// Set the keyboard LED suspend animation to enabled while the device is suspended
    fn set_sleep_enabled(&mut self, enabled: bool) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_states_enabled(ctrl.config.awake_enabled, enabled)
                .map_err(|err| warn!("{}", err))
                .ok();
            ctrl.config.sleep_anim_enabled = enabled;
            ctrl.config.write();
            let states = LedPowerStates {
                enabled: ctrl.config.awake_enabled,
                sleep_anim_enabled: ctrl.config.sleep_anim_enabled,
            };
            self.notify_power_states(&states)
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    fn set_led_mode(&mut self, effect: AuraEffect) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            match ctrl.do_command(effect) {
                Ok(_) => {
                    if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                        self.notify_led(mode.clone())
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
                Err(err) => {
                    warn!("{}", err);
                }
            }
        }
    }

    fn next_led_mode(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.toggle_mode(false)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                self.notify_led(mode.clone())
                    .unwrap_or_else(|err| warn!("{}", err));
            }
        }
    }

    fn prev_led_mode(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.toggle_mode(true)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                self.notify_led(mode.clone())
                    .unwrap_or_else(|err| warn!("{}", err));
            }
        }
    }

    fn next_led_brightness(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.next_brightness()
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    fn prev_led_brightness(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.prev_brightness()
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    #[dbus_interface(property)]
    fn awake_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.awake_enabled;
        }
        true
    }

    #[dbus_interface(property)]
    fn sleep_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.sleep_anim_enabled;
        }
        true
    }

    /// Return the current mode data
    #[dbus_interface(property)]
    fn led_mode(&self) -> String {
        if let Ok(ctrl) = self.0.try_lock() {
            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                if let Ok(json) = serde_json::to_string(&mode) {
                    return json;
                }
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not deserialise".to_string()
    }

    /// Return a list of available modes
    #[dbus_interface(property)]
    fn led_modes(&self) -> String {
        if let Ok(ctrl) = self.0.try_lock() {
            if let Ok(json) = serde_json::to_string(&ctrl.config.builtins) {
                return json;
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not serialise".to_string()
    }

    /// Return the current LED brightness
    #[dbus_interface(property)]
    fn led_brightness(&self) -> i8 {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.get_brightness().map(|n| n as i8).unwrap_or(-1);
        }
        warn!("SetKeyBacklight could not serialise");
        -1
    }

    #[dbus_interface(signal)]
    fn notify_led(&self, data: AuraEffect) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    fn notify_power_states(&self, data: &LedPowerStates) -> zbus::Result<()>;
}

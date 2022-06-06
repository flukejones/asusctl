use async_trait::async_trait;
use log::warn;
use rog_aura::{AuraEffect, LedBrightness, LedPowerStates};
use zbus::{dbus_interface, Connection, SignalContext};

use super::controller::CtrlKbdLedZbus;

#[async_trait]
impl crate::ZbusAdd for CtrlKbdLedZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Led", server).await;
    }
}

/// The main interface for changing, reading, or notfying signals
///
/// LED commands are split between Brightness, Modes, Per-Key
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlKbdLedZbus {
    /// Set the keyboard brightness level (0-3)
    async fn set_brightness(&mut self, brightness: LedBrightness) {
        if let Ok(ctrl) = self.0.try_lock() {
            ctrl.set_brightness(brightness)
                .map_err(|err| warn!("{}", err))
                .ok();
        }
    }

    /// Set the keyboard LED to enabled while the device is awake
    async fn set_awake_enabled(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut states = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_states_enabled(enabled, ctrl.config.sleep_anim_enabled)
                .map_err(|err| warn!("{}", err))
                .ok();
            ctrl.config.awake_enabled = enabled;
            ctrl.config.write();

            states = Some(LedPowerStates {
                enabled: ctrl.config.awake_enabled,
                sleep_anim_enabled: ctrl.config.sleep_anim_enabled,
            });
        }
        // Need to pull state out like this due to MutexGuard
        if let Some(states) = states {
            Self::notify_power_states(&ctxt, &states)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    /// Set the keyboard LED suspend animation to enabled while the device is suspended
    async fn set_sleep_enabled(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut states = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_states_enabled(ctrl.config.awake_enabled, enabled)
                .map_err(|err| warn!("{}", err))
                .ok();
            ctrl.config.sleep_anim_enabled = enabled;
            ctrl.config.write();
            states = Some(LedPowerStates {
                enabled: ctrl.config.awake_enabled,
                sleep_anim_enabled: ctrl.config.sleep_anim_enabled,
            });
        }
        if let Some(states) = states {
            Self::notify_power_states(&ctxt, &states)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    /// Set the keyboard side LEDs to enabled
    async fn set_side_leds_enabled(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: bool,
    ) {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_side_leds_states(enabled)
                .map_err(|err| warn!("{}", err))
                .ok();
            ctrl.config.side_leds_enabled = enabled;
            ctrl.config.write();
            led = Some(enabled);
        }
        if let Some(led) = led {
            Self::notify_side_leds(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    async fn set_led_mode(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        effect: AuraEffect,
    ) {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            match ctrl.do_command(effect) {
                Ok(_) => {
                    if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                        led = Some(mode.clone());
                    }
                }
                Err(err) => {
                    warn!("{}", err);
                }
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    async fn next_led_mode(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.lock() {
            ctrl.toggle_mode(false)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                led = Some(mode.clone());
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    async fn prev_led_mode(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.lock() {
            ctrl.toggle_mode(true)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                led = Some(mode.clone());
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    async fn next_led_brightness(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.next_brightness()
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    async fn prev_led_brightness(&self) {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.prev_brightness()
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    #[dbus_interface(property)]
    async fn awake_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.awake_enabled;
        }
        true
    }

    #[dbus_interface(property)]
    async fn sleep_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.sleep_anim_enabled;
        }
        true
    }

    #[dbus_interface(property)]
    fn side_leds_enabled(&self) -> bool {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.side_leds_enabled;
        }
        true
    }

    /// Return the current mode data
    #[dbus_interface(property)]
    async fn led_mode(&self) -> String {
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
    async fn led_modes(&self) -> String {
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
    async fn led_brightness(&self) -> i8 {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.get_brightness().map(|n| n as i8).unwrap_or(-1);
        }
        warn!("SetKeyBacklight could not serialise");
        -1
    }

    #[dbus_interface(signal)]
    async fn notify_led(signal_ctxt: &SignalContext<'_>, data: AuraEffect) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn notify_side_leds(signal_ctxt: &SignalContext<'_>, data: bool) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn notify_power_states(
        signal_ctxt: &SignalContext<'_>,
        data: &LedPowerStates,
    ) -> zbus::Result<()>;
}

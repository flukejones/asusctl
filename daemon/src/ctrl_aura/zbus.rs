use async_trait::async_trait;
use log::warn;
use rog_aura::{usb::AuraControl, AuraEffect, LedBrightness};
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

    /// Set a variety of states, input is array of enum.
    ///
    /// enum AuraControl {
    ///     BootLogo,
    ///     BootKeyb,
    ///     AwakeLogo,
    ///     AwakeKeyb,
    ///     SleepLogo,
    ///     SleepKeyb,
    ///     ShutdownLogo,
    ///     ShutdownKeyb,
    ///     AwakeBar,
    ///     BootBar,
    ///     SleepBar,
    ///     ShutdownBar,
    /// }
    async fn set_leds_enabled(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enabled: Vec<AuraControl>,
    ) -> zbus::fdo::Result<()> {
        let mut states = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            for s in enabled {
                ctrl.config.enabled.insert(s);
            }
            ctrl.config.write();

            ctrl.set_power_states(&ctrl.config).map_err(|e| {
                warn!("{}", e);
                e
            })?;

            let set: Vec<AuraControl> = ctrl.config.enabled.iter().map(|v| *v).collect();
            states = Some(set);
        }
        // Need to pull state out like this due to MutexGuard
        if let Some(states) = states {
            Self::notify_power_states(&ctxt, &states)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
        Ok(())
    }

    async fn set_leds_disabled(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        disabled: Vec<AuraControl>,
    ) -> zbus::fdo::Result<()> {
        let mut states = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            for s in disabled {
                ctrl.config.enabled.remove(&s);
            }
            ctrl.config.write();

            ctrl.set_power_states(&ctrl.config).map_err(|e| {
                warn!("{}", e);
                e
            })?;

            let set: Vec<AuraControl> = ctrl.config.enabled.iter().map(|v| *v).collect();
            states = Some(set);
        }
        // Need to pull state out like this due to MutexGuard
        if let Some(states) = states {
            Self::notify_power_states(&ctxt, &states)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
        Ok(())
    }

    async fn set_led_mode(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        effect: AuraEffect,
    ) -> zbus::fdo::Result<()> {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.set_effect(effect).map_err(|e| {
                warn!("{}", e);
                e
            })?;
            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                led = Some(mode.clone());
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
        Ok(())
    }

    async fn next_led_mode(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.lock() {
            ctrl.toggle_mode(false).map_err(|e| {
                warn!("{}", e);
                e
            })?;

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                led = Some(mode.clone());
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
        Ok(())
    }

    async fn prev_led_mode(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        let mut led = None;
        if let Ok(mut ctrl) = self.0.lock() {
            ctrl.toggle_mode(true).map_err(|e| {
                warn!("{}", e);
                e
            })?;

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                led = Some(mode.clone());
            }
        }
        if let Some(led) = led {
            Self::notify_led(&ctxt, led)
                .await
                .unwrap_or_else(|err| warn!("{}", err));
        }
        Ok(())
    }

    async fn next_led_brightness(&self) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.next_brightness().map_err(|e| {
                warn!("{}", e);
                e
            })?;
        }
        Ok(())
    }

    async fn prev_led_brightness(&self) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.prev_brightness().map_err(|e| {
                warn!("{}", e);
                e
            })?;
        }
        Ok(())
    }

    #[dbus_interface(property)]
    async fn leds_enabled(&self) -> Vec<u8> {
        if let Ok(ctrl) = self.0.try_lock() {
            let set: Vec<AuraControl> = ctrl.config.enabled.iter().map(|v| *v).collect();
            return AuraControl::to_bytes(&set).to_vec();
        }
        vec![0, 0]
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
    async fn notify_power_states(
        signal_ctxt: &SignalContext<'_>,
        data: &[AuraControl],
    ) -> zbus::Result<()>;
}

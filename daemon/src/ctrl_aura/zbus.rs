use std::collections::BTreeMap;

use async_trait::async_trait;
use log::warn;
use rog_aura::{usb::AuraPowerDev, AuraEffect, AuraModeNum, LedBrightness};
use zbus::{dbus_interface, Connection, SignalContext};

use super::controller::CtrlKbdLedZbus;

#[async_trait]
impl crate::ZbusAdd for CtrlKbdLedZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Aura", server).await;
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
    /// `enabled` sets if the sent array should be disabled or enabled
    ///
    /// ```text
    /// pub struct AuraPowerDev {
    ///     pub x1866: Vec<AuraDev1866>,
    ///     pub x19b6: Vec<AuraDev19b6>,
    /// }
    /// pub enum AuraDev1866 {
    ///     Awake,
    ///     Keyboard,
    ///     Lightbar,
    ///     Boot,
    ///     Sleep,
    /// }
    /// enum AuraDev19b6 {
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
    /// ```
    async fn set_leds_power(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        options: AuraPowerDev,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        let mut states = None;
        if let Ok(mut ctrl) = self.0.try_lock() {
            for p in options.tuf {
                ctrl.config.enabled.set_tuf(p, enabled);
            }
            for p in options.x1866 {
                ctrl.config.enabled.set_0x1866(p, enabled);
            }
            for p in options.x19b6 {
                ctrl.config.enabled.set_0x19b6(p, enabled);
            }

            ctrl.config.write();

            ctrl.set_power_states().map_err(|e| {
                warn!("{}", e);
                e
            })?;

            states = Some(AuraPowerDev::from(&ctrl.config.enabled));
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

    // As property doesn't work for AuraPowerDev (complexity of serialization?)
    // #[dbus_interface(property)]
    async fn leds_enabled(&self) -> AuraPowerDev {
        loop {
            if let Ok(ctrl) = self.0.try_lock() {
                return AuraPowerDev::from(&ctrl.config.enabled);
            }
        }
    }

    /// Return the current mode data
    async fn led_mode(&self) -> AuraModeNum {
        if let Ok(ctrl) = self.0.try_lock() {
            return ctrl.config.current_mode;
        }
        AuraModeNum::Static
    }

    /// Return a list of available modes
    async fn led_modes(&self) -> BTreeMap<AuraModeNum, AuraEffect> {
        loop {
            if let Ok(ctrl) = self.0.try_lock() {
                return ctrl.config.builtins.clone();
            }
        }
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
        data: &AuraPowerDev,
    ) -> zbus::Result<()>;
}

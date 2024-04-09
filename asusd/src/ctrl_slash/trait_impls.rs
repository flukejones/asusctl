use std::sync::Arc;

use config_traits::StdConfig;
use log::warn;
use rog_slash::usb::{pkt_set_mode, pkt_set_options};
use rog_slash::{DeviceState, SlashMode};
use zbus::export::futures_util::lock::Mutex;
use zbus::{interface, Connection, SignalContext};

use crate::ctrl_slash::CtrlSlash;
use crate::error::RogError;

pub const SLASH_ZBUS_NAME: &str = "Slash";
pub const SLASH_ZBUS_PATH: &str = "/org/asuslinux";

#[derive(Clone)]
pub struct CtrlSlashZbus(pub Arc<Mutex<CtrlSlash>>);

/// The struct with the main dbus methods requires this trait
impl crate::ZbusRun for CtrlSlashZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, SLASH_ZBUS_PATH, server).await;
    }
}

#[interface(name = "org.asuslinux.Slash")]
impl CtrlSlashZbus {
    /// Get enabled or not
    #[zbus(property)]
    async fn enabled(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.slash_enabled
    }

    /// Set enabled true or false
    async fn set_enabled(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        let brightness = if enabled && lock.config.slash_brightness == 0 {
            0x88
        } else {
            lock.config.slash_brightness
        };
        lock.node
            .write_bytes(&pkt_set_options(
                enabled,
                brightness,
                lock.config.slash_interval,
            ))
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_enabled = enabled;
        lock.config.slash_brightness = brightness;
        lock.config.write();
    }

    /// Get brightness level
    #[zbus(property)]
    async fn brightness(&self) -> u8 {
        let lock = self.0.lock().await;
        lock.config.slash_brightness
    }

    /// Set brightness level
    async fn set_brightness(&self, brightness: u8) {
        let mut lock = self.0.lock().await;
        let enabled = brightness > 0;
        lock.node
            .write_bytes(&pkt_set_options(
                enabled,
                brightness,
                lock.config.slash_interval,
            ))
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_enabled = enabled;
        lock.config.slash_brightness = brightness;
        lock.config.write();
    }

    #[zbus(property)]
    async fn interval(&self) -> u8 {
        let lock = self.0.lock().await;
        lock.config.slash_interval
    }

    /// Set interval between slash animations (0-255)
    async fn set_interval(&self, interval: u8) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_options(
                lock.config.slash_enabled,
                lock.config.slash_brightness,
                interval,
            ))
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_interval = interval;
        lock.config.write();
    }

    #[zbus(property)]
    async fn slash_mode(&self) -> u8 {
        let lock = self.0.lock().await;
        lock.config.slash_interval
    }

    /// Set interval between slash animations (0-255)
    async fn set_slash_mode(&self, slash_mode: SlashMode) {
        let mut lock = self.0.lock().await;

        let command_packets = pkt_set_mode(slash_mode);

        lock.node
            .write_bytes(&command_packets[0])
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();
        lock.node
            .write_bytes(&command_packets[1])
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_mode = slash_mode;
        lock.config.write();
    }

    /// Get the device state as stored by asusd
    // #[zbus(property)]
    async fn device_state(&self) -> DeviceState {
        let lock = self.0.lock().await;
        DeviceState::from(&lock.config)
    }
}

impl crate::CtrlTask for CtrlSlashZbus {
    fn zbus_path() -> &'static str {
        SLASH_ZBUS_PATH
    }

    async fn create_tasks(&self, _: SignalContext<'static>) -> Result<(), RogError> {
        Ok(())
    }
}

impl crate::Reloadable for CtrlSlashZbus {
    async fn reload(&mut self) -> Result<(), RogError> {
        Ok(())
    }
}

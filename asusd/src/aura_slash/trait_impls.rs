use config_traits::StdConfig;
use log::{debug, error, warn};
use rog_slash::usb::{pkt_save, pkt_set_mode, pkt_set_options};
use rog_slash::{DeviceState, SlashMode};
use zbus::zvariant::OwnedObjectPath;
use zbus::{interface, Connection};

use super::Slash;
use crate::error::RogError;
use crate::Reloadable;

#[derive(Clone)]
pub struct SlashZbus(Slash);

impl SlashZbus {
    pub fn new(slash: Slash) -> Self {
        Self(slash)
    }

    pub async fn start_tasks(
        mut self,
        connection: &Connection,
        path: OwnedObjectPath,
    ) -> Result<(), RogError> {
        // let task = zbus.clone();
        self.reload()
            .await
            .unwrap_or_else(|err| warn!("Controller error: {}", err));
        connection
            .object_server()
            .at(path.clone(), self)
            .await
            .map_err(|e| error!("Couldn't add server at path: {path}, {e:?}"))
            .ok();
        Ok(())
    }
}

#[interface(name = "xyz.ljones.Slash")]
impl SlashZbus {
    /// Get enabled or not
    #[zbus(property)]
    async fn enabled(&self) -> bool {
        let lock = self.0.lock_config().await;
        lock.slash_enabled
    }

    /// Set enabled true or false
    #[zbus(property)]
    async fn set_enabled(&self, enabled: bool) {
        let mut config = self.0.lock_config().await;
        let brightness = if enabled && config.slash_brightness == 0 {
            0x88
        } else {
            config.slash_brightness
        };
        self.0
            .write_bytes(&pkt_set_options(
                config.slash_type,
                enabled,
                brightness,
                config.slash_interval,
            ))
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        config.slash_enabled = enabled;
        config.slash_brightness = brightness;
        config.write();
    }

    /// Get brightness level
    #[zbus(property)]
    async fn brightness(&self) -> u8 {
        let config = self.0.lock_config().await;
        config.slash_brightness
    }

    /// Set brightness level
    #[zbus(property)]
    async fn set_brightness(&self, brightness: u8) {
        let mut config = self.0.lock_config().await;
        let enabled = brightness > 0;
        self.0
            .write_bytes(&pkt_set_options(
                config.slash_type,
                enabled,
                brightness,
                config.slash_interval,
            ))
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        config.slash_enabled = enabled;
        config.slash_brightness = brightness;
        config.write();
    }

    #[zbus(property)]
    async fn interval(&self) -> u8 {
        let config = self.0.lock_config().await;
        config.slash_interval
    }

    /// Set interval between slash animations (0-255)
    #[zbus(property)]
    async fn set_interval(&self, interval: u8) {
        let mut config = self.0.lock_config().await;
        self.0
            .write_bytes(&pkt_set_options(
                config.slash_type,
                config.slash_enabled,
                config.slash_brightness,
                interval,
            ))
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        config.slash_interval = interval;
        config.write();
    }

    #[zbus(property)]
    async fn slash_mode(&self) -> u8 {
        let config = self.0.lock_config().await;
        config.slash_interval
    }

    /// Set interval between slash animations (0-255)
    #[zbus(property)]
    async fn set_slash_mode(&self, slash_mode: SlashMode) {
        let mut config = self.0.lock_config().await;

        let command_packets = pkt_set_mode(config.slash_type, slash_mode);
        // self.node.write_bytes(&command_packets[0])?;
        self.0
            .write_bytes(&command_packets[1])
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();
        self.0
            .write_bytes(&pkt_save(config.slash_type))
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        config.slash_mode = slash_mode;
        config.write();
    }

    /// Get the device state as stored by asusd
    // #[zbus(property)]
    async fn device_state(&self) -> DeviceState {
        let config = self.0.lock_config().await;
        DeviceState::from(&*config)
    }
}

impl Reloadable for SlashZbus {
    async fn reload(&mut self) -> Result<(), RogError> {
        debug!("reloading slash settings");
        let config = self.0.lock_config().await;
        self.0
            .write_bytes(&pkt_set_options(
                config.slash_type,
                config.slash_enabled,
                config.slash_brightness,
                config.slash_interval,
            ))
            .await
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();
        Ok(())
    }
}

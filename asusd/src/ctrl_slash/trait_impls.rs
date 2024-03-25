use std::sync::Arc;
use log::warn;
use zbus::{Connection, interface};
use zbus::export::futures_util::lock::Mutex;
use config_traits::StdConfig;
use rog_slash::DeviceState;
use rog_slash::usb::pkt_set_options;
use crate::ctrl_slash::CtrlSlash;


pub const SLASH_ZBUS_NAME: &str = "Slash";
pub const SLASH_ZBUS_PATH: &str = "/org/asuslinux";

#[derive(Clone)]
pub struct CtrlSlashZbus(pub Arc<Mutex<CtrlSlash>>);


/// The struct with the main dbus methods requires this trait
impl crate::ZbusRun for CtrlSlashZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, SLASH_ZBUS_NAME, server).await;
    }
}


// None of these calls can be guarnateed to succeed unless we loop until okay
// If the try_lock *does* succeed then any other thread trying to lock will not
// grab it until we finish.
#[interface(name = "org.asuslinux.Slash")]
impl CtrlSlashZbus {
    // /// Writes a data stream of length. Will force system thread to exit until
    // /// it is restarted
    // async fn write(&self, input: AnimeDataBuffer) -> zbus::fdo::Result<()> {
    //     let lock = self.0.lock().await;
    //     lock.thread_exit.store(true, Ordering::SeqCst);
    //     lock.write_data_buffer(input).map_err(|err| {
    //         warn!("ctrl_anime::run_animation:callback {}", err);
    //         err
    //     })?;
    //     Ok(())
    // }

    /// Set base brightness level
    #[zbus(property)]
    async fn brightness(&self) -> u8 {
        let lock = self.0.lock().await;
        lock.config.slash_brightness
    }

    /// Set base brightness level
    #[zbus(property)]
    async fn set_brightness(&self, brightness: u8) {
        let mut lock = self.0.lock().await;
        let enabled = brightness > 0;
        lock.node
            .write_bytes(&pkt_set_options(enabled, brightness, lock.config.slash_interval))
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_enabled = enabled;
        lock.config.slash_brightness = brightness;
        lock.config.write();
    }

    #[zbus(property)]
    async fn enable_display(&self) -> bool {
        let lock = self.0.lock().await;
        lock.config.slash_enabled
    }

    /// Set whether the AniMe is enabled at all
    #[zbus(property)]
    async fn set_enable_display(&self, enabled: bool) {
        let mut lock = self.0.lock().await;
        lock.node
            .write_bytes(&pkt_set_options(enabled, lock.config.slash_brightness, lock.config.slash_interval))
            .map_err(|err| {
                warn!("ctrl_slash::set_options {}", err);
            })
            .ok();

        lock.config.slash_enabled = enabled;
        lock.config.write();
    }

    /// Get the device state as stored by asusd
    // #[zbus(property)]
    async fn device_state(&self) -> DeviceState {
        let lock = self.0.lock().await;
        DeviceState::from(&lock.config)
    }
}
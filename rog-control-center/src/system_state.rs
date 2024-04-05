use std::sync::{Arc, Mutex};

use log::error;
use supergfxctl::pci_device::{GfxMode, GfxPower};
#[cfg(not(feature = "mocking"))]
use supergfxctl::zbus_proxy::DaemonProxy as GfxProxy;
use zbus::Connection;

use crate::error::Result;
#[cfg(feature = "mocking")]
use crate::mocking::DaemonProxyBlocking as GfxProxyBlocking;
use crate::update_and_notify::EnabledNotifications;

#[derive(Clone, Debug)]
pub struct GfxState {
    pub has_supergfx: bool,
    pub mode: GfxMode,
    pub power_status: GfxPower,
}

impl GfxState {
    pub async fn new(dbus: &GfxProxy<'_>) -> Result<Self> {
        Ok(Self {
            has_supergfx: dbus.mode().await.is_ok(),
            mode: dbus.mode().await.unwrap_or(GfxMode::None),
            power_status: dbus.power().await.unwrap_or(GfxPower::Unknown),
        })
    }
}

impl Default for GfxState {
    fn default() -> Self {
        Self {
            has_supergfx: false,
            mode: GfxMode::None,
            power_status: GfxPower::Unknown,
        }
    }
}

///  State stored from system daemons. This is shared with: tray, zbus
/// notifications thread and the GUI app thread.
pub struct SystemState {
    pub enabled_notifications: Arc<Mutex<EnabledNotifications>>,
    pub gfx_state: GfxState,
    pub error: Option<String>,
    /// Specific field for the tray only so that we can know when it does need
    /// update. The tray should set this to false when done.
    pub tray_should_update: bool,
    pub app_should_update: bool,
    pub tray_enabled: bool,
    pub run_in_bg: bool,
}

impl SystemState {
    /// Creates self, including the relevant dbus connections and proixies for
    /// internal use
    pub async fn new(
        enabled_notifications: Arc<Mutex<EnabledNotifications>>,
        tray_enabled: bool,
        run_in_bg: bool,
    ) -> Result<Self> {
        let conn = Connection::system().await?;

        let gfx_dbus = GfxProxy::builder(&conn)
            .destination(":org.supergfxctl.Daemon")?
            .build()
            .await
            .expect("Couldn't connect to supergfxd");
        Ok(Self {
            enabled_notifications,
            gfx_state: GfxState::new(&gfx_dbus)
                .await
                .map_err(|e| {
                    let e = format!("Could not get supergfxd state: {e}");
                    error!("{e}");
                })
                .unwrap_or_default(),
            error: None,
            tray_should_update: true,
            app_should_update: true,
            tray_enabled,
            run_in_bg,
        })
    }

    pub fn set_notified(&mut self) {
        self.tray_should_update = true;
        self.app_should_update = true;
    }
}

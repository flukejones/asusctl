use ::zbus::dbus_interface;
use log::{error, info, warn};
use rog_types::gfx_vendors::{GfxPower, GfxRequiredUserAction, GfxVendors};
use zvariant::ObjectPath;

use crate::ZbusAdd;

use super::controller::CtrlGraphics;

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlGraphics {
    fn vendor(&self) -> zbus::fdo::Result<GfxVendors> {
        self.get_gfx_mode().map_err(|err| {
            error!("GFX: {}", err);
            zbus::fdo::Error::Failed(format!("GFX fail: {}", err))
        })
    }

    fn power(&self) -> zbus::fdo::Result<GfxPower> {
        Self::get_runtime_status().map_err(|err| {
            error!("GFX: {}", err);
            zbus::fdo::Error::Failed(format!("GFX fail: {}", err))
        })
    }

    fn set_vendor(&mut self, vendor: GfxVendors) -> zbus::fdo::Result<GfxRequiredUserAction> {
        info!("GFX: Switching gfx mode to {}", <&str>::from(vendor));
        let msg = self.set_gfx_mode(vendor).map_err(|err| {
            error!("GFX: {}", err);
            zbus::fdo::Error::Failed(format!("GFX fail: {}", err))
        })?;
        self.notify_gfx(&vendor)
            .unwrap_or_else(|err| warn!("GFX: {}", err));
        self.notify_action(&msg)
            .unwrap_or_else(|err| warn!("GFX: {}", err));
        Ok(msg)
    }

    #[dbus_interface(signal)]
    fn notify_gfx(&self, vendor: &GfxVendors) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    fn notify_action(&self, action: &GfxRequiredUserAction) -> zbus::Result<()> {}
}

impl ZbusAdd for CtrlGraphics {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&ObjectPath::from_str_unchecked("/org/asuslinux/Gfx"), self)
            .map_err(|err| {
                warn!("GFX: CtrlGraphics: add_to_server {}", err);
                err
            })
            .ok();
    }
}

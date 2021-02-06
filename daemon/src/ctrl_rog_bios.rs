use crate::{
    config::Config,
    ctrl_gfx::{gfx::CtrlGraphics},
    error::RogError,
    GetSupported,
};
//use crate::dbus::DbusEvents;
use log::{info, warn};
use rog_types::gfx_vendors::GfxVendors;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;

static ASUS_SWITCH_GRAPHIC_MODE: &str =
    "/sys/firmware/efi/efivars/AsusSwitchGraphicMode-607005d5-3f75-4b2e-98f0-85ba66797a3e";
static ASUS_POST_LOGO_SOUND: &str =
    "/sys/firmware/efi/efivars/AsusPostLogoSound-607005d5-3f75-4b2e-98f0-85ba66797a3e";

pub struct CtrlRogBios {
    _config: Arc<Mutex<Config>>,
}

#[derive(Serialize, Deserialize)]
pub struct RogBiosSupportedFunctions {
    pub post_sound_toggle: bool,
    pub dedicated_gfx_toggle: bool,
}

impl GetSupported for CtrlRogBios {
    type A = RogBiosSupportedFunctions;

    fn get_supported() -> Self::A {
        RogBiosSupportedFunctions {
            post_sound_toggle: CtrlRogBios::check_path_exists(ASUS_POST_LOGO_SOUND).is_ok(),
            dedicated_gfx_toggle: CtrlRogBios::check_path_exists(ASUS_SWITCH_GRAPHIC_MODE).is_ok(),
        }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlRogBios {
    pub fn set_dedicated_graphic_mode(&mut self, dedicated: bool) {
        Self::set_gfx_mode(dedicated)
            .map_err(|err| {
                warn!("CtrlRogBios: set_asus_switch_graphic_mode {}", err);
                err
            })
            .ok();
        self.notify_dedicated_graphic_mode(dedicated)
            .map_err(|err| {
                warn!("CtrlRogBios: notify_asus_switch_graphic_mode {}", err);
                err
            })
            .ok();
    }

    pub fn dedicated_graphic_mode(&self) -> i8 {
        Self::get_gfx_mode()
            .map_err(|err| {
                warn!("CtrlRogBios: get_gfx_mode {}", err);
                err
            })
            .unwrap_or(-1)
    }

    #[dbus_interface(signal)]
    pub fn notify_dedicated_graphic_mode(&self, dedicated: bool) -> zbus::Result<()> {}

    // // // // // // // // // //

    pub fn set_post_boot_sound(&mut self, on: bool) {
        Self::set_boot_sound(on)
            .map_err(|err| {
                warn!("CtrlRogBios: set_post_boot_sound {}", err);
                err
            })
            .ok();
        self.notify_post_boot_sound(on)
            .map_err(|err| {
                warn!("CtrlRogBios: notify_post_boot_sound {}", err);
                err
            })
            .ok();
    }

    pub fn post_boot_sound(&self) -> i8 {
        Self::get_boot_sound()
            .map_err(|err| {
                warn!("CtrlRogBios: get_boot_sound {}", err);
                err
            })
            .unwrap_or(-1)
    }

    #[dbus_interface(signal)]
    pub fn notify_post_boot_sound(&self, dedicated: bool) -> zbus::Result<()> {}
}

impl crate::ZbusAdd for CtrlRogBios {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&"/org/asuslinux/RogBios".try_into().unwrap(), self)
            .map_err(|err| {
                warn!("CtrlRogBios: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl crate::Reloadable for CtrlRogBios {
    fn reload(&mut self) -> Result<(), RogError> {
        Ok(())
    }
}

impl CtrlRogBios {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        match CtrlRogBios::check_path_exists(ASUS_SWITCH_GRAPHIC_MODE) {
            Ok(_) => {
                CtrlRogBios::set_path_mutable(ASUS_SWITCH_GRAPHIC_MODE)?;
            }
            Err(err) => {
                info!("ROG Switchable Graphics (bios) not detected: {}", err);
            }
        }

        match CtrlRogBios::check_path_exists(ASUS_POST_LOGO_SOUND) {
            Ok(_) => {
                CtrlRogBios::set_path_mutable(ASUS_POST_LOGO_SOUND)?;
            }
            Err(err) => {
                info!("ROG boot sound toggle (bios) not detected: {}", err);
            }
        }

        Ok(CtrlRogBios { _config: config })
    }

    fn set_path_mutable(path: &str) -> Result<(), RogError> {
        use std::process::Command;

        let output = Command::new("/usr/bin/chattr")
            .arg("-i")
            .arg(path)
            .output()
            .map_err(|err| RogError::Path(path.into(), err))?;
        info!("Set {} writeable: status: {}", path, output.status);
        Ok(())
    }

    fn check_path_exists(path: &str) -> Result<(), RogError> {
        if Path::new(path).exists() {
            Ok(())
        } else {
            Err(RogError::MissingFunction(path.into()))
        }
    }

    pub fn has_dedicated_gfx_toggle() -> bool {
        if CtrlRogBios::check_path_exists(ASUS_SWITCH_GRAPHIC_MODE).is_ok() {
            return true;
        }
        false
    }

    pub fn get_gfx_mode() -> Result<i8, RogError> {
        let path = ASUS_SWITCH_GRAPHIC_MODE;
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|err| RogError::Path(path.into(), err))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|err| RogError::Read(path.into(), err))?;

        let idx = data.len() - 1;
        Ok(data[idx] as i8)
    }

    pub(super) fn set_gfx_mode(dedicated: bool) -> Result<(), RogError> {
        let path = ASUS_SWITCH_GRAPHIC_MODE;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| RogError::Path(path.into(), err))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let idx = data.len() - 1;
        if dedicated {
            data[idx] = 1;
            info!("Set system-level graphics mode: Dedicated Nvidia");
        } else {
            data[idx] = 0;
            info!("Set system-level graphics mode: Optimus");
        }
        file.write_all(&data)
            .map_err(|err| RogError::Path(path.into(), err))?;

        if let Ok(ded) = CtrlRogBios::get_gfx_mode() {
            if let Ok(vendor) = CtrlGraphics::get_vendor() {
                if ded == 1 && vendor != "nvidia" {
                    warn!("Dedicated GFX toggle is on but driver mode is not nvidia \nSetting to nvidia driver mode");
                    CtrlGraphics::set_gfx_config(GfxVendors::Nvidia)
                        .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
                }
            }
        }

        Ok(())
    }

    pub fn get_boot_sound() -> Result<i8, RogError> {
        let path = ASUS_POST_LOGO_SOUND;
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|err| RogError::Path(path.into(), err))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|err| RogError::Read(path.into(), err))?;

        let idx = data.len() - 1;
        Ok(data[idx] as i8)
    }

    pub(super) fn set_boot_sound(on: bool) -> Result<(), RogError> {
        let path = ASUS_POST_LOGO_SOUND;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| RogError::Path(path.into(), err))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|err| RogError::Read(path.into(), err))?;

        let idx = data.len() - 1;
        if on {
            data[idx] = 1;
            info!("Set boot POST sound on");
        } else {
            data[idx] = 0;
            info!("Set boot POST sound off");
        }
        file.write_all(&data)
            .map_err(|err| RogError::Path(path.into(), err))?;

        Ok(())
    }
}

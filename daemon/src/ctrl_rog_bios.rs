use crate::{config::Config, error::RogError, GetSupported};
use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;
use zvariant::ObjectPath;

const INITRAMFS_PATH: &str = "/usr/sbin/update-initramfs";
const DRACUT_PATH: &str = "/usr/bin/dracut";

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
        self.set_gfx_mode(dedicated)
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
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/RogBios"),
                self,
            )
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

    pub(super) fn set_gfx_mode(&self, dedicated: bool) -> Result<(), RogError> {
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

        self.update_initramfs(dedicated)?;

        // if let Ok(ded) = CtrlRogBios::get_gfx_mode() {
        //     if let Ok(vendor) = CtrlGraphics::get_vendor() {
        //         if ded == 1 && vendor != "nvidia" {
        //             warn!("Dedicated GFX toggle is on but driver mode is not nvidia \nSetting to nvidia driver mode");
        //             CtrlGraphics::set_gfx_config(&GfxVendors::Nvidia)
        //                 .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
        //         }
        //     }
        // }

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

    // required for g-sync mode
    fn update_initramfs(&self, dedicated: bool) -> Result<(), RogError> {
        let mut initfs_cmd = None;

        if Path::new(INITRAMFS_PATH).exists() {
            let mut cmd = Command::new("update-initramfs");
            cmd.arg("-u");
            initfs_cmd = Some(cmd);
            info!("Using initramfs update command 'update-initramfs'");
        } else if Path::new(DRACUT_PATH).exists() {
            let mut cmd = Command::new("dracut");
            cmd.arg("-f");
            cmd.arg("-q");
            initfs_cmd = Some(cmd);
            info!("Using initramfs update command 'dracut'");
        }

        if let Some(mut cmd) = initfs_cmd {
            info!("Updating initramfs");

            // If switching to Nvidia dedicated we need these modules included
            if Path::new(DRACUT_PATH).exists() && dedicated {
                cmd.arg("--add-drivers");
                cmd.arg("nvidia nvidia-drm nvidia-modeset nvidia-uvm");
                info!("System uses dracut, forcing nvidia modules to be included in init");
            } else if Path::new(INITRAMFS_PATH).exists() {
                let modules = vec![
                    "nvidia\n",
                    "nvidia-drm\n",
                    "nvidia-modeset\n",
                    "nvidia-uvm\n",
                ];

                let module_include = Path::new("/etc/initramfs-tools/modules");

                if dedicated {
                    let mut file = std::fs::OpenOptions::new()
                        .append(true)
                        .open(module_include)
                        .map_err(|err| {
                            RogError::Write(module_include.to_string_lossy().to_string(), err)
                        })?;
                    // add nvidia modules to module_include
                    file.write_all(modules.concat().as_bytes()).unwrap();
                } else {
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .open(module_include)
                        .map_err(|err| {
                            RogError::Write(module_include.to_string_lossy().to_string(), err)
                        })?;

                    let mut buf = Vec::new();
                    // remove modules
                    for line in std::io::BufReader::new(file).lines() {
                        if let Ok(l) = line {
                            if !modules.contains(&l.as_ref()) {
                                buf.append(&mut l.as_bytes().to_vec());
                            }
                        }
                    }

                    let file = std::fs::OpenOptions::new()
                        .write(true)
                        .open(module_include)
                        .map_err(|err| {
                            RogError::Write(module_include.to_string_lossy().to_string(), err)
                        })?;
                    std::io::BufWriter::new(file).write_all(&buf).unwrap();
                }
            }

            let status = cmd
                .status()
                .map_err(|err| RogError::Write(format!("{:?}", cmd), err))?;
            if !status.success() {
                error!("Ram disk update failed");
                return Err(RogError::Initramfs("Ram disk update failed".into()));
            } else {
                info!("Successfully updated initramfs");
            }
        }
        Ok(())
    }
}

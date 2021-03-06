use crate::{config::Config, error::RogError, GetSupported};
use async_trait::async_trait;
use log::{error, info, warn};
use rog_supported::RogBiosSupportedFunctions;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::Connection;
use zbus::{dbus_interface, SignalContext};

const INITRAMFS_PATH: &str = "/usr/sbin/update-initramfs";
const DRACUT_PATH: &str = "/usr/bin/dracut";

static ASUS_SWITCH_GRAPHIC_MODE: &str =
    "/sys/firmware/efi/efivars/AsusSwitchGraphicMode-607005d5-3f75-4b2e-98f0-85ba66797a3e";
static ASUS_POST_LOGO_SOUND: &str =
    "/sys/firmware/efi/efivars/AsusPostLogoSound-607005d5-3f75-4b2e-98f0-85ba66797a3e";
static ASUS_PANEL_OD_PATH: &str = "/sys/devices/platform/asus-nb-wmi/panel_od";
static ASUS_DGPU_DISABLE_PATH: &str = "/sys/devices/platform/asus-nb-wmi/dgpu_disable";
static ASUS_EGPU_ENABLE_PATH: &str = "/sys/devices/platform/asus-nb-wmi/egpu_enable";

pub struct CtrlRogBios {
    _config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlRogBios {
    type A = RogBiosSupportedFunctions;

    fn get_supported() -> Self::A {
        RogBiosSupportedFunctions {
            post_sound: Path::new(ASUS_POST_LOGO_SOUND).exists(),
            dedicated_gfx: Path::new(ASUS_SWITCH_GRAPHIC_MODE).exists(),
            panel_overdrive: Path::new(ASUS_PANEL_OD_PATH).exists(),
            dgpu_disable: Path::new(ASUS_DGPU_DISABLE_PATH).exists(),
            egpu_enable: Path::new(ASUS_EGPU_ENABLE_PATH).exists(),
        }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlRogBios {
    async fn set_dedicated_graphic_mode(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        dedicated: bool,
    ) {
        self.set_gfx_mode(dedicated)
            .map_err(|err| {
                warn!("CtrlRogBios: set_asus_switch_graphic_mode {}", err);
                err
            })
            .ok();
        Self::notify_dedicated_graphic_mode(&ctxt, dedicated)
            .await
            .ok();
    }

    fn dedicated_graphic_mode(&self) -> i8 {
        Self::get_gfx_mode()
            .map_err(|err| {
                warn!("CtrlRogBios: get_gfx_mode {}", err);
                err
            })
            .unwrap_or(-1)
    }

    #[dbus_interface(signal)]
    async fn notify_dedicated_graphic_mode(
        signal_ctxt: &SignalContext<'_>,
        dedicated: bool,
    ) -> zbus::Result<()> {
    }

    async fn set_post_boot_sound(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        on: bool,
    ) {
        Self::set_boot_sound(on)
            .map_err(|err| {
                warn!("CtrlRogBios: set_post_boot_sound {}", err);
                err
            })
            .ok();
        Self::notify_post_boot_sound(&ctxt, on).await.ok();
    }

    fn post_boot_sound(&self) -> i8 {
        Self::get_boot_sound()
            .map_err(|err| {
                warn!("CtrlRogBios: get_boot_sound {}", err);
                err
            })
            .unwrap_or(-1)
    }

    #[dbus_interface(signal)]
    async fn notify_post_boot_sound(ctxt: &SignalContext<'_>, on: bool) -> zbus::Result<()> {}

    async fn set_panel_overdrive(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        overdrive: bool,
    ) {
        if self
            .set_panel_od(overdrive)
            .map_err(|err| {
                warn!("CtrlRogBios: set_panel_overdrive {}", err);
                err
            })
            .is_ok()
        {
            Self::notify_panel_overdrive(&ctxt, overdrive).await.ok();
        }
    }

    fn panel_overdrive(&self) -> i8 {
        let path = ASUS_PANEL_OD_PATH;
        if let Ok(mut file) = OpenOptions::new().read(true).open(path).map_err(|err| {
            warn!("CtrlRogBios: panel_overdrive {}", err);
            err
        }) {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|err| {
                    warn!("CtrlRogBios: set_panel_overdrive {}", err);
                    err
                })
                .ok();

            if buf.len() >= 1 {
                let tmp = String::from_utf8_lossy(&buf[0..1]);
                return tmp.parse::<i8>().unwrap_or(-1);
            }
        }
        -1
    }

    #[dbus_interface(signal)]
    async fn notify_panel_overdrive(
        signal_ctxt: &SignalContext<'_>,
        overdrive: bool,
    ) -> zbus::Result<()> {
    }
}

#[async_trait]
impl crate::ZbusAdd for CtrlRogBios {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/RogBios", server).await;
    }
}

impl crate::Reloadable for CtrlRogBios {
    fn reload(&mut self) -> Result<(), RogError> {
        Ok(())
    }
}

impl CtrlRogBios {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        if Path::new(ASUS_SWITCH_GRAPHIC_MODE).exists() {
            CtrlRogBios::set_path_mutable(ASUS_SWITCH_GRAPHIC_MODE)?;
        } else {
            info!("G-Sync Switchable Graphics not detected");
            info!("If your laptop is not a G-Sync enabled laptop then you can ignore this. Standard graphics switching will still work.");
        }

        if Path::new(ASUS_POST_LOGO_SOUND).exists() {
            CtrlRogBios::set_path_mutable(ASUS_POST_LOGO_SOUND)?;
        } else {
            info!("Switch for POST boot sound not detected");
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

    pub fn has_dedicated_gfx_toggle() -> bool {
        Path::new(ASUS_SWITCH_GRAPHIC_MODE).exists()
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
        file.read_to_end(&mut data)?;

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
                    file.write_all(modules.concat().as_bytes())?;
                } else {
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .open(module_include)
                        .map_err(|err| {
                            RogError::Write(module_include.to_string_lossy().to_string(), err)
                        })?;

                    let mut buf = Vec::new();
                    // remove modules
                    for line in std::io::BufReader::new(file).lines().flatten() {
                        if !modules.contains(&line.as_str()) {
                            buf.append(&mut line.as_bytes().to_vec());
                        }
                    }

                    let file = std::fs::OpenOptions::new()
                        .write(true)
                        .open(module_include)
                        .map_err(|err| {
                            RogError::Write(module_include.to_string_lossy().to_string(), err)
                        })?;
                    std::io::BufWriter::new(file).write_all(&buf)?;
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

    fn set_panel_od(&mut self, overdrive: bool) -> Result<(), RogError> {
        let path = ASUS_PANEL_OD_PATH;
        let mut file = OpenOptions::new().write(true).open(path).map_err(|err| {
            warn!("CtrlRogBios: set_panel_overdrive {}", err);
            err
        })?;

        let s = if overdrive { '1' } else { '0' };
        file.write(&[s as u8]).map_err(|err| {
            warn!("CtrlRogBios: set_panel_overdrive {}", err);
            err
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CtrlRogBios;
    use crate::config::Config;
    use std::sync::{Arc, Mutex};

    #[test]
    #[ignore = "Must be manually tested"]
    fn set_multizone_4key_config() {
        let config = Config::default();

        let controller = CtrlRogBios {
            _config: Arc::new(Mutex::new(config)),
        };

        let res = controller.panel_overdrive();
        assert_eq!(res, 1);

        // controller.set_panel_od(false).unwrap();
        // let res = controller.panel_overdrive();
        // assert_eq!(res, 0);
    }
}

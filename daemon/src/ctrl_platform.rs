use crate::CtrlTask;
use crate::{config::Config, error::RogError, GetSupported};
use async_trait::async_trait;
use log::{info, warn};
use rog_platform::platform::{AsusPlatform, GpuMode};
use rog_platform::supported::RogBiosSupportedFunctions;
use smol::Executor;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::Connection;
use zbus::{dbus_interface, SignalContext};

static ASUS_POST_LOGO_SOUND: &str =
    "/sys/firmware/efi/efivars/AsusPostLogoSound-607005d5-3f75-4b2e-98f0-85ba66797a3e";

#[derive(Clone)]
pub struct CtrlRogBios {
    platform: AsusPlatform,
    config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlRogBios {
    type A = RogBiosSupportedFunctions;

    fn get_supported() -> Self::A {
        let mut panel_overdrive = false;
        let mut dgpu_disable = false;
        let mut egpu_enable = false;
        let mut gpu_mux = false;

        if let Ok(platform) = AsusPlatform::new() {
            panel_overdrive = platform.has_panel_od();
            dgpu_disable = platform.has_dgpu_disable();
            egpu_enable = platform.has_egpu_enable();
            gpu_mux = platform.has_gpu_mux_mode();
        }

        RogBiosSupportedFunctions {
            post_sound: Path::new(ASUS_POST_LOGO_SOUND).exists(),
            gpu_mux,
            panel_overdrive,
            dgpu_disable,
            egpu_enable,
        }
    }
}

impl CtrlRogBios {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let platform = AsusPlatform::new()?;

        if !platform.has_gpu_mux_mode() {
            info!("G-Sync Switchable Graphics or GPU MUX not detected");
            info!("Standard graphics switching will still work.");
        }

        if Path::new(ASUS_POST_LOGO_SOUND).exists() {
            CtrlRogBios::set_path_mutable(ASUS_POST_LOGO_SOUND)?;
        } else {
            info!("Switch for POST boot sound not detected");
        }

        Ok(CtrlRogBios { platform, config })
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

    fn set_gfx_mode(&self, mode: GpuMode) -> Result<(), RogError> {
        self.platform.set_gpu_mux_mode(mode.to_mux())?;
        // self.update_initramfs(enable)?;
        if mode == GpuMode::Discrete {
            info!("Set system-level graphics mode: Dedicated Nvidia");
        } else {
            info!("Set system-level graphics mode: Optimus");
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

    fn set_panel_od(&self, enable: bool) -> Result<(), RogError> {
        self.platform.set_panel_od(enable).map_err(|err| {
            warn!("CtrlRogBios: set_panel_overdrive {}", err);
            err
        })?;
        Ok(())
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlRogBios {
    async fn set_gpu_mux_mode(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        mode: GpuMode,
    ) {
        self.set_gfx_mode(mode)
            .map_err(|err| {
                warn!("CtrlRogBios: set_asus_switch_graphic_mode {}", err);
                err
            })
            .ok();
        Self::notify_gpu_mux_mode(&ctxt, mode).await.ok();
    }

    fn gpu_mux_mode(&self) -> GpuMode {
        match self.platform.get_gpu_mux_mode() {
            Ok(m) => GpuMode::from_mux(m),
            Err(e) => {
                warn!("CtrlRogBios: get_gfx_mode {}", e);
                GpuMode::Error
            }
        }
    }

    #[dbus_interface(signal)]
    async fn notify_gpu_mux_mode(
        signal_ctxt: &SignalContext<'_>,
        mode: GpuMode,
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
            if let Ok(mut lock) = self.config.try_lock() {
                lock.panel_od = overdrive;
                lock.write();
            }
            Self::notify_panel_overdrive(&ctxt, overdrive).await.ok();
        }
    }

    fn panel_overdrive(&self) -> bool {
        self.platform
            .get_panel_od()
            .map_err(|err| {
                warn!("CtrlRogBios: get panel overdrive {}", err);
                err
            })
            .unwrap_or(false)
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
        Self::add_to_server_helper(self, "/org/asuslinux/Platform", server).await;
    }
}

impl crate::Reloadable for CtrlRogBios {
    fn reload(&mut self) -> Result<(), RogError> {
        if self.platform.has_panel_od() {
            let p = if let Ok(lock) = self.config.try_lock() {
                lock.panel_od
            } else {
                false
            };
            self.set_panel_od(p)?;
        }
        Ok(())
    }
}

#[async_trait]
impl CtrlTask for CtrlRogBios {
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let platform1 = self.clone();
        let platform2 = self.clone();
        self.create_sys_event_tasks(
            executor,
            move || {},
            move || {
                info!("CtrlRogBios reloading panel_od");
                if let Ok(lock) = platform1.config.try_lock() {
                    if platform1.platform.has_panel_od() {
                        platform1
                            .set_panel_od(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: set_limit {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
            move || {},
            move || {
                info!("CtrlRogBios reloading panel_od");
                if let Ok(lock) = platform2.config.try_lock() {
                    if platform2.platform.has_panel_od() {
                        platform2
                            .set_panel_od(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: set_limit {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
        )
        .await;

        Ok(())
    }
}

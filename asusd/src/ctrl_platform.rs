use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::{info, warn};
use rog_platform::platform::{AsusPlatform, GpuMode};
use rog_platform::supported::RogBiosSupportedFunctions;
use zbus::export::futures_util::lock::Mutex;
use zbus::{dbus_interface, Connection, SignalContext};

use crate::config::Config;
use crate::error::RogError;
use crate::{task_watch_item, CtrlTask, GetSupported};

const ZBUS_PATH: &str = "/org/asuslinux/Platform";
const ASUS_POST_LOGO_SOUND: &str =
    "/sys/firmware/efi/efivars/AsusPostLogoSound-607005d5-3f75-4b2e-98f0-85ba66797a3e";

#[derive(Clone)]
pub struct CtrlPlatform {
    platform: AsusPlatform,
    config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlPlatform {
    type A = RogBiosSupportedFunctions;

    fn get_supported() -> Self::A {
        let mut panel_overdrive = false;
        let mut mini_led_mode = false;
        let mut dgpu_disable = false;
        let mut egpu_enable = false;
        let mut gpu_mux = false;

        if let Ok(platform) = AsusPlatform::new() {
            panel_overdrive = platform.has_panel_od();
            mini_led_mode = platform.has_mini_led_mode();
            dgpu_disable = platform.has_dgpu_disable();
            egpu_enable = platform.has_egpu_enable();
            gpu_mux = platform.has_gpu_mux_mode();
        }

        RogBiosSupportedFunctions {
            post_sound: Path::new(ASUS_POST_LOGO_SOUND).exists(),
            gpu_mux,
            panel_overdrive,
            mini_led_mode,
            dgpu_disable,
            egpu_enable,
        }
    }
}

impl CtrlPlatform {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let platform = AsusPlatform::new()?;

        if !platform.has_gpu_mux_mode() {
            info!("G-Sync Switchable Graphics or GPU MUX not detected");
            info!("Standard graphics switching will still work.");
        }

        if Path::new(ASUS_POST_LOGO_SOUND).exists() {
            CtrlPlatform::set_path_mutable(ASUS_POST_LOGO_SOUND)?;
        } else {
            info!("Switch for POST boot sound not detected");
        }

        Ok(CtrlPlatform { platform, config })
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
        self.platform.set_gpu_mux_mode(mode.to_mux_attr())?;
        // self.update_initramfs(enable)?;
        if mode == GpuMode::Discrete {
            info!("Set system-level graphics mode: Dedicated Nvidia");
        } else {
            info!("Set system-level graphics mode: Optimus");
        }
        Ok(())
    }

    pub fn get_boot_sound() -> Result<i8, RogError> {
        let data = std::fs::read(ASUS_POST_LOGO_SOUND)
            .map_err(|err| RogError::Read(ASUS_POST_LOGO_SOUND.into(), err))?;

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
        #[allow(clippy::verbose_file_reads)]
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

    fn set_panel_overdrive(&self, enable: bool) -> Result<(), RogError> {
        self.platform.set_panel_od(enable).map_err(|err| {
            warn!("CtrlRogBios: set_panel_overdrive {}", err);
            err
        })?;
        Ok(())
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlPlatform {
    async fn set_gpu_mux_mode(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        mode: GpuMode,
    ) {
        self.set_gfx_mode(mode)
            .map_err(|err| {
                warn!("CtrlRogBios: set_gpu_mux_mode {}", err);
                err
            })
            .ok();
        Self::notify_gpu_mux_mode(&ctxt, mode).await.ok();
    }

    fn gpu_mux_mode(&self) -> GpuMode {
        match self.platform.get_gpu_mux_mode() {
            Ok(m) => GpuMode::from_mux(m as u8),
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
        Self::notify_post_boot_sound(&ctxt, on)
            .await
            .map_err(|err| {
                warn!("CtrlRogBios: set_post_boot_sound {}", err);
                err
            })
            .ok();
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

    async fn set_panel_od(&mut self, overdrive: bool) {
        match self.platform.set_panel_od(overdrive) {
            Ok(_) => {
                if let Some(mut lock) = self.config.try_lock() {
                    lock.panel_od = overdrive;
                    lock.write();
                }
            }
            Err(err) => warn!("CtrlRogBios: set_panel_overdrive {}", err),
        };
    }

    /// Get the `panel_od` value from platform. Updates the stored value in
    /// internal config also.
    fn panel_od(&self) -> bool {
        self.platform
            .get_panel_od()
            .map_err(|err| {
                warn!("CtrlRogBios: get_panel_od {}", err);
                err
            })
            .unwrap_or(false)
    }

    #[dbus_interface(signal)]
    async fn notify_panel_od(signal_ctxt: &SignalContext<'_>, overdrive: bool) -> zbus::Result<()> {
    }

    async fn set_mini_led_mode(&mut self, on: bool) {
        match self.platform.set_mini_led_mode(on) {
            Ok(_) => {
                if let Some(mut lock) = self.config.try_lock() {
                    lock.mini_led_mode = on;
                    lock.write();
                }
            }
            Err(err) => warn!("CtrlRogBios: set_mini_led_mode {}", err),
        };
    }

    /// Get the `panel_od` value from platform. Updates the stored value in
    /// internal config also.
    fn mini_led_mode(&self) -> bool {
        self.platform
            .get_mini_led_mode()
            .map_err(|err| {
                warn!("CtrlRogBios: get_mini_led_mode {}", err);
                err
            })
            .unwrap_or(false)
    }

    #[dbus_interface(signal)]
    async fn notify_mini_led_mode(signal_ctxt: &SignalContext<'_>, on: bool) -> zbus::Result<()> {}

    async fn set_dgpu_disable(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        disable: bool,
    ) {
        match self.platform.set_dgpu_disable(disable) {
            Ok(_) => {
                Self::notify_dgpu_disable(&ctxt, disable).await.ok();
            }
            Err(err) => warn!("CtrlRogBios: set_dgpu_disable {}", err),
        };
    }

    fn dgpu_disable(&self) -> bool {
        self.platform
            .get_dgpu_disable()
            .map_err(|err| {
                warn!("CtrlRogBios: get_dgpu_disable {}", err);
                err
            })
            .unwrap_or(false)
    }

    #[dbus_interface(signal)]
    async fn notify_dgpu_disable(
        signal_ctxt: &SignalContext<'_>,
        disable: bool,
    ) -> zbus::Result<()> {
    }

    async fn set_egpu_enable(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        enable: bool,
    ) {
        match self.platform.set_egpu_enable(enable) {
            Ok(_) => {
                Self::notify_egpu_enable(&ctxt, enable).await.ok();
            }
            Err(err) => warn!("CtrlRogBios: set_egpu_enable {}", err),
        };
    }

    fn egpu_enable(&self) -> bool {
        self.platform
            .get_egpu_enable()
            .map_err(|err| {
                warn!("CtrlRogBios: get_egpu_enable {}", err);
                err
            })
            .unwrap_or(false)
    }

    #[dbus_interface(signal)]
    async fn notify_egpu_enable(signal_ctxt: &SignalContext<'_>, enable: bool) -> zbus::Result<()> {
    }
}

#[async_trait]
impl crate::ZbusRun for CtrlPlatform {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Platform", server).await;
    }
}

#[async_trait]
impl crate::Reloadable for CtrlPlatform {
    async fn reload(&mut self) -> Result<(), RogError> {
        if self.platform.has_panel_od() {
            let p = if let Some(lock) = self.config.try_lock() {
                lock.panel_od
            } else {
                false
            };
            self.set_panel_overdrive(p)?;
        }
        Ok(())
    }
}

impl CtrlPlatform {
    task_watch_item!(panel_od platform);

    task_watch_item!(dgpu_disable platform);

    task_watch_item!(egpu_enable platform);

    task_watch_item!(mini_led_mode platform);
    // NOTE: see note further below
    // task_watch_item!(gpu_mux_mode platform);
}

#[async_trait]
impl CtrlTask for CtrlPlatform {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let platform1 = self.clone();
        let platform2 = self.clone();
        self.create_sys_event_tasks(
            move || async { {} },
            move || {
                let platform1 = platform1.clone();
                async move {
                    info!("CtrlRogBios reloading panel_od");
                    let lock = platform1.config.lock().await;
                    if platform1.platform.has_panel_od() {
                        platform1
                            .set_panel_overdrive(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: panel_od {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
            move || async { {} },
            move || {
                let platform2 = platform2.clone();
                async move {
                    info!("CtrlRogBios reloading panel_od");
                    let lock = platform2.config.lock().await;
                    if platform2.platform.has_panel_od() {
                        platform2
                            .set_panel_overdrive(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: panel_od {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
        )
        .await;

        self.watch_panel_od(signal_ctxt.clone()).await?;
        self.watch_dgpu_disable(signal_ctxt.clone()).await?;
        self.watch_egpu_enable(signal_ctxt.clone()).await?;
        self.watch_mini_led_mode(signal_ctxt.clone()).await?;
        // NOTE: Can't have this as a watch because on a write to it, it reverts back to
        // booted-with value  as it does not actually change until reboot.
        // self.watch_gpu_mux_mode(signal_ctxt.clone()).await?;

        Ok(())
    }
}

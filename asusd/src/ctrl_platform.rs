use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::{error, info, warn};
use rog_platform::platform::{AsusPlatform, GpuMode};
use rog_platform::supported::PlatformSupportedFunctions;
use zbus::export::futures_util::lock::Mutex;
use zbus::fdo::Error as FdoErr;
use zbus::{dbus_interface, Connection, SignalContext};

use crate::config::Config;
use crate::error::RogError;
use crate::{task_watch_item, CtrlTask, GetSupported};

const ZBUS_PATH: &str = "/org/asuslinux/Platform";

macro_rules! platform_get_value {
    ($self:ident, $property:tt, $prop_name:literal) => {
        concat_idents::concat_idents!(has = has_, $property {
            if $self.platform.has() {
                concat_idents::concat_idents!(get = get_, $property {
                    $self.platform
                    .get()
                    .map_err(|err| {
                        warn!("CtrlRogBios: {}: {}", $prop_name, err);
                        FdoErr::Failed(format!("CtrlRogBios: {}: {}", $prop_name, err))
                    })
                })
            } else {
                error!("CtrlRogBios: {} not supported", $prop_name);
                return Err(FdoErr::NotSupported(format!("CtrlRogBios: {} not supported", $prop_name)));
            }
        })
    }
}

macro_rules! platform_get_value_if_some {
    ($self:ident, $property:tt, $prop_name:literal, $default:expr) => {
        concat_idents::concat_idents!(has = has_, $property {
            if $self.platform.has() {
                let lock = $self.config.lock().await;
                Ok(lock.ppt_pl1_spl.unwrap_or($default))
            } else {
                error!("CtrlRogBios: {} not supported", $prop_name);
                return Err(FdoErr::NotSupported(format!("CtrlRogBios: {} not supported", $prop_name)));
            }
        })
    }
}

macro_rules! platform_set_bool {
    ($self:ident, $property:tt, $prop_name:literal, $new_value:expr) => {
        concat_idents::concat_idents!(has = has_, $property {
            if $self.platform.has() {
                concat_idents::concat_idents!(set = set_, $property {
                    $self.platform.set($new_value).map_err(|err| {
                        error!("CtrlRogBios: {} {err}", $prop_name);
                        FdoErr::NotSupported(format!("CtrlRogBios: {} {err}", $prop_name))
                    })?;
                });
                let mut lock = $self.config.lock().await;
                lock.$property = $new_value;
                lock.write();
                Ok(())
            } else {
                error!("CtrlRogBios: {} not supported", $prop_name);
                Err(FdoErr::NotSupported(format!("CtrlRogBios: {} not supported", $prop_name)))
            }
        })
    }
}

/// Intended only for setting platform object values where the value isn't
/// retained across boots
macro_rules! platform_set_with_min_max {
    ($self:ident, $property:tt, $prop_name:literal, $new_value:expr, $min_value:expr, $max_value:expr) => {
        if !($min_value..=$max_value).contains(&$new_value) {
            Err(FdoErr::Failed(
                format!("CtrlRogBios: {} value not in range {}=..={}", $prop_name, $min_value, $max_value)
            ))
        } else {
            concat_idents::concat_idents!(has = has_, $property {
                if $self.platform.has() {
                    concat_idents::concat_idents!(set = set_, $property {
                        $self.platform.set($new_value).map_err(|err| {
                            error!("CtrlRogBios: {} {err}", $prop_name);
                            FdoErr::NotSupported(format!("CtrlRogBios: {} {err}", $prop_name))
                        })?;
                    });
                    let mut lock = $self.config.lock().await;
                    lock.$property = Some($new_value);
                    lock.write();
                } else {
                    error!("CtrlRogBios: {} not supported", $prop_name);
                    return Err(FdoErr::NotSupported(format!("CtrlRogBios: {} not supported", $prop_name)));
                }
            });
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct CtrlPlatform {
    platform: AsusPlatform,
    config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlPlatform {
    type A = PlatformSupportedFunctions;

    fn get_supported() -> Self::A {
        let platform = AsusPlatform::new().unwrap_or_default();
        platform.into()
    }
}

impl CtrlPlatform {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let platform = AsusPlatform::new()?;

        if !platform.has_gpu_mux_mode() {
            info!("G-Sync Switchable Graphics or GPU MUX not detected");
            info!("Standard graphics switching will still work.");
        }

        Ok(CtrlPlatform { platform, config })
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
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlPlatform {
    /// Returns a list of property names that this system supports
    fn supported_properties(&self) -> Vec<String> {
        let mut supported = Vec::new();

        macro_rules! push_name {
            ($property:tt, $prop_name:literal) => {
                concat_idents::concat_idents!(has = has_, $property {
                    if self.platform.has() {
                        supported.push($prop_name.to_owned());
                    }
                })
            }
        }

        push_name!(dgpu_disable, "dgpu_disable");
        push_name!(gpu_mux_mode, "gpu_mux_mode");
        push_name!(post_animation_sound, "post_animation_sound");
        push_name!(panel_od, "panel_od");
        push_name!(mini_led_mode, "mini_led_mode");
        push_name!(egpu_enable, "egpu_enable");

        push_name!(ppt_pl1_spl, "ppt_pl1_spl");
        push_name!(ppt_pl2_sppt, "ppt_pl2_sppt");
        push_name!(ppt_fppt, "ppt_fppt");
        push_name!(ppt_apu_sppt, "ppt_apu_sppt");
        push_name!(ppt_platform_sppt, "ppt_platform_sppt");
        push_name!(nv_dynamic_boost, "nv_dynamic_boost");
        push_name!(nv_temp_target, "nv_temp_target");

        supported
    }

    #[dbus_interface(property)]
    fn gpu_mux_mode(&self) -> Result<u8, FdoErr> {
        self.platform.get_gpu_mux_mode().map_err(|err| {
            warn!("CtrlRogBios: set_gpu_mux_mode {err}");
            FdoErr::NotSupported("CtrlRogBios: set_gpu_mux_mode not supported".to_owned())
        })
    }

    #[dbus_interface(property)]
    async fn set_gpu_mux_mode(&mut self, mode: u8) -> Result<(), FdoErr> {
        if self.platform.has_gpu_mux_mode() {
            self.set_gfx_mode(mode.into()).map_err(|err| {
                warn!("CtrlRogBios: set_gpu_mux_mode {}", err);
                FdoErr::Failed(format!("CtrlRogBios: set_gpu_mux_mode: {err}"))
            })
        } else {
            Err(FdoErr::NotSupported(
                "CtrlRogBios: set_gpu_mux_mode not supported".to_owned(),
            ))
        }
    }

    #[dbus_interface(property)]
    fn post_animation_sound(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, post_animation_sound, "post_animation_sound")
    }

    #[dbus_interface(property)]
    async fn set_post_animation_sound(&mut self, on: bool) -> Result<(), FdoErr> {
        platform_set_bool!(self, post_animation_sound, "post_animation_sound", on)
    }

    /// Get the `panel_od` value from platform. Updates the stored value in
    /// internal config also.
    #[dbus_interface(property)]
    fn panel_od(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, panel_od, "panel_od")
    }

    #[dbus_interface(property)]
    async fn set_panel_od(&mut self, overdrive: bool) -> Result<(), FdoErr> {
        platform_set_bool!(self, panel_od, "panel_od", overdrive)
    }

    /// Get the `panel_od` value from platform. Updates the stored value in
    /// internal config also.
    #[dbus_interface(property)]
    fn mini_led_mode(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, mini_led_mode, "mini_led_mode")
    }

    #[dbus_interface(property)]
    async fn set_mini_led_mode(&mut self, on: bool) -> Result<(), FdoErr> {
        platform_set_bool!(self, mini_led_mode, "mini_led_mode", on)
    }

    #[dbus_interface(property)]
    fn dgpu_disable(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, dgpu_disable, "dgpu_disable")
    }

    #[dbus_interface(property)]
    fn egpu_enable(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, egpu_enable, "egpu_enable")
    }

    /// ************************************************************************
    #[dbus_interface(property)]
    async fn ppt_pl1_spl(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, ppt_pl1_spl, "ppt_pl1_spl", 5)
    }

    #[dbus_interface(property)]
    async fn set_ppt_pl1_spl(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, ppt_pl1_spl, "ppt_pl1_spl", value, 5, 250)
    }

    #[dbus_interface(property)]
    async fn ppt_pl2_sppt(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, ppt_pl2_sppt, "ppt_pl2_sppt", 5)
    }

    #[dbus_interface(property)]
    async fn set_ppt_pl2_sppt(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, ppt_pl2_sppt, "ppt_pl2_sppt", value, 5, 250)
    }

    #[dbus_interface(property)]
    async fn ppt_fppt(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, ppt_fppt, "ppt_fppt", 5)
    }

    #[dbus_interface(property)]
    async fn set_ppt_fppt(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, ppt_fppt, "ppt_fppt", value, 5, 250)
    }

    #[dbus_interface(property)]
    async fn ppt_apu_sppt(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, ppt_apu_sppt, "ppt_apu_sppt", 5)
    }

    #[dbus_interface(property)]
    async fn set_ppt_apu_sppt(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, ppt_apu_sppt, "ppt_apu_sppt", value, 5, 130)
    }

    #[dbus_interface(property)]
    async fn ppt_platform_sppt(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, ppt_platform_sppt, "ppt_platform_sppt", 5)
    }

    #[dbus_interface(property)]
    async fn set_ppt_platform_sppt(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, ppt_platform_sppt, "ppt_platform_sppt", value, 5, 130)
    }

    #[dbus_interface(property)]
    async fn nv_dynamic_boost(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, nv_dynamic_boost, "nv_dynamic_boost", 5)
    }

    #[dbus_interface(property)]
    async fn set_nv_dynamic_boost(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, nv_dynamic_boost, "nv_dynamic_boost", value, 5, 25)
    }

    #[dbus_interface(property)]
    async fn nv_temp_target(&self) -> Result<u8, FdoErr> {
        platform_get_value_if_some!(self, nv_temp_target, "nv_temp_target", 5)
    }

    #[dbus_interface(property)]
    async fn set_nv_temp_target(&mut self, value: u8) -> Result<(), FdoErr> {
        platform_set_with_min_max!(self, nv_temp_target, "nv_temp_target", value, 5, 87)
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
            self.platform.set_panel_od(p)?;
        }
        Ok(())
    }
}

impl CtrlPlatform {
    task_watch_item!(panel_od platform);
    // task_watch_item!(dgpu_disable platform);
    // task_watch_item!(egpu_enable platform);
    // task_watch_item!(mini_led_mode platform);
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
            move |sleeping| {
                let platform1 = platform1.clone();
                async move {
                    info!("CtrlRogBios reloading panel_od");
                    let lock = platform1.config.lock().await;
                    if !sleeping && platform1.platform.has_panel_od() {
                        platform1
                            .platform
                            .set_panel_od(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: panel_od {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
            move |shutting_down| {
                let platform2 = platform2.clone();
                async move {
                    info!("CtrlRogBios reloading panel_od");
                    let lock = platform2.config.lock().await;
                    if !shutting_down && platform2.platform.has_panel_od() {
                        platform2
                            .platform
                            .set_panel_od(lock.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: panel_od {}", err);
                                err
                            })
                            .ok();
                    }
                }
            },
            move |_lid_closed| {
                // on lid change
                async move {}
            },
            move |_power_plugged| {
                // power change
                async move {}
            },
        )
        .await;

        self.watch_panel_od(signal_ctxt.clone()).await?;
        // self.watch_dgpu_disable(signal_ctxt.clone()).await?;
        // self.watch_egpu_enable(signal_ctxt.clone()).await?;
        // self.watch_mini_led_mode(signal_ctxt.clone()).await?;
        // NOTE: Can't have this as a watch because on a write to it, it reverts back to
        // booted-with value  as it does not actually change until reboot.
        // self.watch_gpu_mux_mode(signal_ctxt.clone()).await?;

        Ok(())
    }
}

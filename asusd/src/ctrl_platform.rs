use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::{debug, error, info, warn};
use rog_platform::cpu::{CPUControl, CPUGovernor};
use rog_platform::platform::{GpuMode, PlatformPolicy, Properties, RogPlatform};
use rog_platform::power::AsusPower;
use zbus::export::futures_util::lock::Mutex;
use zbus::fdo::Error as FdoErr;
use zbus::{dbus_interface, Connection, ObjectServer, SignalContext};

use crate::config::Config;
use crate::ctrl_anime::trait_impls::{CtrlAnimeZbus, ANIME_ZBUS_NAME, ANIME_ZBUS_PATH};
use crate::ctrl_aura::trait_impls::{CtrlAuraZbus, AURA_ZBUS_NAME, AURA_ZBUS_PATH};
use crate::ctrl_fancurves::{CtrlFanCurveZbus, FAN_CURVE_ZBUS_NAME, FAN_CURVE_ZBUS_PATH};
use crate::error::RogError;
use crate::{task_watch_item, task_watch_item_notify, CtrlTask};

const ZBUS_PATH: &str = "/org/asuslinux/Platform";

macro_rules! platform_get_value {
    ($self:ident, $property:tt, $prop_name:literal) => {
        concat_idents::concat_idents!(has = has_, $property {
            if $self.platform.has() {
                concat_idents::concat_idents!(get = get_, $property {
                    $self.platform
                    .get()
                    .map_err(|err| {
                        warn!("RogPlatform: {}: {}", $prop_name, err);
                        FdoErr::Failed(format!("RogPlatform: {}: {}", $prop_name, err))
                    })
                })
            } else {
                info!("RogPlatform: {} not supported", $prop_name);
                return Err(FdoErr::NotSupported(format!("RogPlatform: {} not supported", $prop_name)));
            }
        })
    }
}

macro_rules! platform_get_value_if_some {
    ($self:ident, $property:tt, $prop_name:literal, $default:expr) => {
        concat_idents::concat_idents!(has = has_, $property {
            if $self.platform.has() {
                let lock = $self.config.lock().await;
                Ok(lock.$property.unwrap_or($default))
            } else {
                info!("RogPlatform: {} not supported", $prop_name);
                return Err(FdoErr::NotSupported(format!("RogPlatform: {} not supported", $prop_name)));
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
                        error!("RogPlatform: {} {err}", $prop_name);
                        FdoErr::NotSupported(format!("RogPlatform: {} {err}", $prop_name))
                    })?;
                });
                let mut lock = $self.config.lock().await;
                lock.$property = $new_value;
                lock.write();
                Ok(())
            } else {
                info!("RogPlatform: {} not supported", $prop_name);
                Err(FdoErr::NotSupported(format!("RogPlatform: {} not supported", $prop_name)))
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
                format!("RogPlatform: {} value not in range {}=..={}", $prop_name, $min_value, $max_value)
            ))
        } else {
            concat_idents::concat_idents!(has = has_, $property {
                if $self.platform.has() {
                    concat_idents::concat_idents!(set = set_, $property {
                        $self.platform.set($new_value).map_err(|err| {
                            error!("RogPlatform: {} {err}", $prop_name);
                            FdoErr::NotSupported(format!("RogPlatform: {} {err}", $prop_name))
                        })?;
                    });
                    let mut lock = $self.config.lock().await;
                    lock.$property = Some($new_value);
                    lock.write();
                } else {
                    error!("RogPlatform: {} not supported", $prop_name);
                    return Err(FdoErr::NotSupported(format!("RogPlatform: {} not supported", $prop_name)));
                }
            });
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct CtrlPlatform {
    power: AsusPower,
    platform: RogPlatform,
    cpu_control: Option<CPUControl>,
    config: Arc<Mutex<Config>>,
}

impl CtrlPlatform {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let platform = RogPlatform::new()?;
        let power = AsusPower::new()?;

        if !platform.has_gpu_mux_mode() {
            info!("G-Sync Switchable Graphics or GPU MUX not detected");
            info!("Standard graphics switching will still work.");
        }

        Ok(CtrlPlatform {
            power,
            platform,
            config,
            cpu_control: CPUControl::new()
                .map_err(|e| error!("Couldn't get CPU control sysfs: {e}"))
                .ok(),
        })
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

    async fn run_ac_or_bat_cmd(&self, power_plugged: bool) {
        let prog: Vec<String> = if power_plugged {
            // AC ONLINE
            self.config
                .lock()
                .await
                .ac_command
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            // BATTERY
            self.config
                .lock()
                .await
                .bat_command
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };
        if prog.len() > 1 {
            let mut cmd = Command::new(&prog[0]);
            for arg in prog.iter().skip(1) {
                cmd.arg(arg);
            }
            if let Err(e) = cmd.spawn() {
                if power_plugged {
                    error!("AC power command error: {e}");
                } else {
                    error!("Battery power command error: {e}");
                }
            }
        }
    }

    fn check_and_set_epp(&self, profile: PlatformPolicy, change_epp: bool) {
        if !change_epp {
            info!("PlatformPolicy unlinked from EPP");
            return;
        }
        info!("PlatformPolicy setting EPP");
        if let Some(cpu) = self.cpu_control.as_ref() {
            if let Ok(epp) = cpu.get_available_epp() {
                debug!("Available EPP: {epp:?}");
                if epp.contains(&profile.into()) {
                    debug!("Setting {profile:?}");
                    cpu.set_epp(profile.into()).ok();
                } else if let Ok(gov) = cpu.get_governor() {
                    if gov != CPUGovernor::Powersave {
                        warn!("powersave governor is not is use, you should use it.");
                    }
                }
            }
        }
    }

    async fn update_policy_ac_or_bat(&self, power_plugged: bool, change_epp: bool) {
        let profile = if power_plugged {
            self.config.lock().await.platform_policy_on_ac
        } else {
            self.config.lock().await.platform_policy_on_battery
        };
        self.platform
            .set_throttle_thermal_policy(profile.into())
            .ok();
        self.check_and_set_epp(profile, change_epp);
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlPlatform {
    /// Returns a list of property names that this system supports
    async fn supported_properties(&self) -> Vec<Properties> {
        let mut supported = Vec::new();

        macro_rules! platform_name {
            ($property:tt, $prop_name:ty) => {
                concat_idents::concat_idents!(has = has_, $property {
                    if self.platform.has() {
                        supported.push($prop_name.to_owned());
                    }
                })
            }
        }

        macro_rules! power_name {
            ($property:tt, $prop_name:ty) => {
                concat_idents::concat_idents!(has = has_, $property {
                    if self.power.has() {
                        supported.push($prop_name.to_owned());
                    }
                })
            }
        }

        // TODO: automate this
        power_name!(
            charge_control_end_threshold,
            Properties::ChargeControlEndThreshold
        );

        platform_name!(dgpu_disable, Properties::DgpuDisable);
        platform_name!(gpu_mux_mode, Properties::GpuMuxMode);
        platform_name!(post_animation_sound, Properties::PostAnimationSound);
        platform_name!(panel_od, Properties::PanelOd);
        platform_name!(mini_led_mode, Properties::MiniLedMode);
        platform_name!(egpu_enable, Properties::EgpuEnable);
        platform_name!(throttle_thermal_policy, Properties::PlatformPolicy);

        platform_name!(ppt_pl1_spl, Properties::PptPl1Spl);
        platform_name!(ppt_pl2_sppt, Properties::PptPl2Sppt);
        platform_name!(ppt_fppt, Properties::PptFppt);
        platform_name!(ppt_apu_sppt, Properties::PptApuSppt);
        platform_name!(ppt_platform_sppt, Properties::PptPlatformSppt);
        platform_name!(nv_dynamic_boost, Properties::NvDynamicBoost);
        platform_name!(nv_temp_target, Properties::NvTempTarget);

        supported
    }

    async fn supported_interfaces(
        &self,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> Vec<String> {
        let mut interfaces = Vec::default();
        if server
            .interface::<_, CtrlAnimeZbus>(ANIME_ZBUS_PATH)
            .await
            .is_ok()
        {
            interfaces.push(ANIME_ZBUS_NAME.to_owned());
        }
        if server
            .interface::<_, CtrlAuraZbus>(AURA_ZBUS_PATH)
            .await
            .is_ok()
        {
            interfaces.push(AURA_ZBUS_NAME.to_owned());
        }
        if server
            .interface::<_, CtrlFanCurveZbus>(FAN_CURVE_ZBUS_PATH)
            .await
            .is_ok()
        {
            interfaces.push(FAN_CURVE_ZBUS_NAME.to_owned());
        }
        interfaces
    }

    #[dbus_interface(property)]
    fn charge_control_end_threshold(&self) -> Result<u8, FdoErr> {
        let limit = self.power.get_charge_control_end_threshold()?;
        Ok(limit)
    }

    #[dbus_interface(property)]
    async fn set_charge_control_end_threshold(&mut self, limit: u8) -> Result<(), FdoErr> {
        if !(20..=100).contains(&limit) {
            return Err(RogError::ChargeLimit(limit))?;
        }
        self.power.set_charge_control_end_threshold(limit)?;
        self.config.lock().await.charge_control_end_threshold = limit;
        Ok(())
    }

    #[dbus_interface(property)]
    fn gpu_mux_mode(&self) -> Result<u8, FdoErr> {
        self.platform.get_gpu_mux_mode().map_err(|err| {
            warn!("RogPlatform: set_gpu_mux_mode {err}");
            FdoErr::NotSupported("RogPlatform: set_gpu_mux_mode not supported".to_owned())
        })
    }

    #[dbus_interface(property)]
    async fn set_gpu_mux_mode(&mut self, mode: u8) -> Result<(), FdoErr> {
        if self.platform.has_gpu_mux_mode() {
            self.set_gfx_mode(mode.into()).map_err(|err| {
                warn!("RogPlatform: set_gpu_mux_mode {}", err);
                FdoErr::Failed(format!("RogPlatform: set_gpu_mux_mode: {err}"))
            })
        } else {
            Err(FdoErr::NotSupported(
                "RogPlatform: set_gpu_mux_mode not supported".to_owned(),
            ))
        }
    }

    /// Toggle to next platform_profile. Names provided by `Profiles`.
    /// If fan-curves are supported will also activate a fan curve for profile.
    async fn next_throttle_thermal_policy(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> Result<(), FdoErr> {
        let policy: PlatformPolicy =
            platform_get_value!(self, throttle_thermal_policy, "throttle_thermal_policy")
                .map(|n| n.into())?;
        let policy = PlatformPolicy::next(&policy);

        if self.platform.has_throttle_thermal_policy() {
            let change_epp = self.config.lock().await.platform_policy_linked_epp;
            self.check_and_set_epp(policy, change_epp);
            self.platform
                .set_throttle_thermal_policy(policy.into())
                .map_err(|err| {
                    warn!("RogPlatform: throttle_thermal_policy {}", err);
                    FdoErr::Failed(format!("RogPlatform: throttle_thermal_policy: {err}"))
                })?;
            Ok(self.throttle_thermal_policy_changed(&ctxt).await?)
        } else {
            Err(FdoErr::NotSupported(
                "RogPlatform: throttle_thermal_policy not supported".to_owned(),
            ))
        }
    }

    #[dbus_interface(property)]
    fn throttle_thermal_policy(&self) -> Result<PlatformPolicy, FdoErr> {
        platform_get_value!(self, throttle_thermal_policy, "throttle_thermal_policy")
            .map(|n| n.into())
    }

    #[dbus_interface(property)]
    async fn set_throttle_thermal_policy(&mut self, policy: PlatformPolicy) -> Result<(), FdoErr> {
        // TODO: watch for external changes
        if self.platform.has_throttle_thermal_policy() {
            let change_epp = self.config.lock().await.platform_policy_linked_epp;
            self.check_and_set_epp(policy, change_epp);
            self.platform
                .set_throttle_thermal_policy(policy.into())
                .map_err(|err| {
                    warn!("RogPlatform: throttle_thermal_policy {}", err);
                    FdoErr::Failed(format!("RogPlatform: throttle_thermal_policy: {err}"))
                })
        } else {
            Err(FdoErr::NotSupported(
                "RogPlatform: throttle_thermal_policy not supported".to_owned(),
            ))
        }
    }

    #[dbus_interface(property)]
    fn post_animation_sound(&self) -> Result<bool, FdoErr> {
        platform_get_value!(self, post_animation_sound, "post_animation_sound")
    }

    #[dbus_interface(property)]
    async fn set_post_animation_sound(&mut self, on: bool) -> Result<(), FdoErr> {
        if self.platform.has_post_animation_sound() {
            self.platform.set_post_animation_sound(on).map_err(|err| {
                warn!("RogPlatform: set_post_animation_sound {}", err);
                FdoErr::Failed(format!("RogPlatform: set_post_animation_sound: {err}"))
            })
        } else {
            Err(FdoErr::NotSupported(
                "RogPlatform: set_post_animation_sound not supported".to_owned(),
            ))
        }
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
        Self::add_to_server_helper(self, ZBUS_PATH, server).await;
    }
}

#[async_trait]
impl crate::Reloadable for CtrlPlatform {
    async fn reload(&mut self) -> Result<(), RogError> {
        if self.platform.has_panel_od() {
            self.platform
                .set_panel_od(self.config.lock().await.panel_od)?;
        }

        if self.platform.has_mini_led_mode() {
            self.platform
                .set_mini_led_mode(self.config.lock().await.mini_led_mode)?;
        }

        if self.power.has_charge_control_end_threshold() {
            self.power.set_charge_control_end_threshold(
                self.config.lock().await.charge_control_end_threshold,
            )?;
        }

        if let Ok(power_plugged) = self.power.get_online() {
            if self.platform.has_throttle_thermal_policy() {
                let change_epp = self.config.lock().await.platform_policy_linked_epp;
                self.update_policy_ac_or_bat(power_plugged > 0, change_epp)
                    .await;
            }
            self.run_ac_or_bat_cmd(power_plugged > 0).await;
        }

        Ok(())
    }
}

impl CtrlPlatform {
    task_watch_item!(panel_od platform);

    task_watch_item!(mini_led_mode platform);

    task_watch_item!(charge_control_end_threshold power);

    task_watch_item_notify!(post_animation_sound platform);

    task_watch_item_notify!(dgpu_disable platform);

    task_watch_item_notify!(egpu_enable platform);

    // NOTE: see note further below
    task_watch_item_notify!(gpu_mux_mode platform);

    task_watch_item_notify!(ppt_pl1_spl platform);

    task_watch_item_notify!(ppt_pl2_sppt platform);

    task_watch_item_notify!(ppt_fppt platform);

    task_watch_item_notify!(ppt_apu_sppt platform);

    task_watch_item_notify!(ppt_platform_sppt platform);

    task_watch_item_notify!(nv_dynamic_boost platform);

    task_watch_item_notify!(nv_temp_target platform);
}

#[async_trait]
impl CtrlTask for CtrlPlatform {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let platform1 = self.clone();
        let platform2 = self.clone();
        let platform3 = self.clone();
        self.create_sys_event_tasks(
            move |sleeping| {
                let platform1 = platform1.clone();
                async move {
                    info!("RogPlatform reloading panel_od");
                    if !sleeping && platform1.platform.has_panel_od() {
                        platform1
                            .platform
                            .set_panel_od(platform1.config.lock().await.panel_od)
                            .map_err(|err| {
                                warn!("CtrlCharge: panel_od {}", err);
                                err
                            })
                            .ok();
                    }
                    if sleeping && platform1.power.has_charge_control_end_threshold() {
                        platform1.config.lock().await.charge_control_end_threshold = platform1
                            .power
                            .get_charge_control_end_threshold()
                            .unwrap_or(100);
                    } else if !sleeping && platform1.power.has_charge_control_end_threshold() {
                        platform1
                            .power
                            .set_charge_control_end_threshold(
                                platform1.config.lock().await.charge_control_end_threshold,
                            )
                            .ok();
                    }
                    if let Ok(power_plugged) = platform1.power.get_online() {
                        if !sleeping && platform1.platform.has_throttle_thermal_policy() {
                            let change_epp =
                                platform1.config.lock().await.platform_policy_linked_epp;
                            platform1
                                .update_policy_ac_or_bat(power_plugged > 0, change_epp)
                                .await;
                        }
                        if !sleeping {
                            platform1.run_ac_or_bat_cmd(power_plugged > 0).await;
                        }
                    }
                }
            },
            move |shutting_down| {
                let platform2 = platform2.clone();
                async move {
                    info!("RogPlatform reloading panel_od");
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
            move |power_plugged| {
                let platform3 = platform3.clone();
                // power change
                async move {
                    if platform3.platform.has_throttle_thermal_policy() {
                        let change_epp = platform3.config.lock().await.platform_policy_linked_epp;
                        platform3
                            .update_policy_ac_or_bat(power_plugged, change_epp)
                            .await;
                    }
                    platform3.run_ac_or_bat_cmd(power_plugged).await;
                }
            },
        )
        .await;

        // This spawns a new task for every item.
        // TODO: find a better way to manage this
        self.watch_panel_od(signal_ctxt.clone()).await?;
        self.watch_mini_led_mode(signal_ctxt.clone()).await?;
        self.watch_charge_control_end_threshold(signal_ctxt.clone())
            .await?;

        self.watch_dgpu_disable(signal_ctxt.clone()).await?;
        self.watch_egpu_enable(signal_ctxt.clone()).await?;

        // NOTE: Can't have this as a watch because on a write to it, it reverts back to
        // booted-with value  as it does not actually change until reboot.
        self.watch_gpu_mux_mode(signal_ctxt.clone()).await?;
        self.watch_post_animation_sound(signal_ctxt.clone()).await?;

        self.watch_ppt_pl1_spl(signal_ctxt.clone()).await?;
        self.watch_ppt_pl2_sppt(signal_ctxt.clone()).await?;
        self.watch_ppt_fppt(signal_ctxt.clone()).await?;
        self.watch_ppt_apu_sppt(signal_ctxt.clone()).await?;
        self.watch_ppt_platform_sppt(signal_ctxt.clone()).await?;
        self.watch_nv_dynamic_boost(signal_ctxt.clone()).await?;
        self.watch_nv_temp_target(signal_ctxt.clone()).await?;

        let watch_throttle_thermal_policy = self.platform.monitor_throttle_thermal_policy()?;
        let ctrl = self.clone();

        tokio::spawn(async move {
            use futures_lite::StreamExt;
            let mut buffer = [0; 32];
            if let Ok(mut stream) = watch_throttle_thermal_policy.into_event_stream(&mut buffer) {
                while (stream.next().await).is_some() {
                    // this blocks
                    debug!("Platform: watch_throttle_thermal_policy changed");
                    if let Ok(profile) = ctrl
                        .platform
                        .get_throttle_thermal_policy()
                        .map(PlatformPolicy::from)
                        .map_err(|e| {
                            error!("Platform: get_throttle_thermal_policy error: {e}");
                        })
                    {
                        let change_epp = ctrl.config.lock().await.platform_policy_linked_epp;
                        ctrl.check_and_set_epp(profile, change_epp);
                    }
                }
            }
        });

        Ok(())
    }
}

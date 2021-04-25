use log::warn;
use rog_fan_curve::Curve;

use std::sync::Arc;
use std::sync::Mutex;
use zbus::{dbus_interface, fdo::Error};
use zvariant::ObjectPath;

use super::CtrlFanAndCpu;

pub struct FanAndCpuZbus {
    inner: Arc<Mutex<CtrlFanAndCpu>>,
}

impl FanAndCpuZbus {
    pub fn new(inner: Arc<Mutex<CtrlFanAndCpu>>) -> Self {
        Self { inner }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl FanAndCpuZbus {
    /// Set profile details
    fn set_profile(&self, profile: String) {
        if let Ok(event) = serde_json::from_str(&profile) {
            if let Ok(mut ctrl) = self.inner.try_lock() {
                if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                    cfg.read();
                    ctrl.handle_profile_event(&event, &mut cfg)
                        .unwrap_or_else(|err| warn!("{}", err));
                    if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                        if let Ok(json) = serde_json::to_string(profile) {
                            self.notify_profile(&json)
                                .unwrap_or_else(|err| warn!("{}", err));
                        }
                    }
                }
            }
        }
    }

    /// Modify the active profile
    fn set_turbo(&self, enable: bool) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                // Update the profile then set it
                cfg.read();
                let profile = cfg.active_profile.clone();
                if let Some(profile) = cfg.power_profiles.get_mut(&profile) {
                    profile.turbo = enable;
                }
                ctrl.set(&profile, &mut cfg)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Modify the active profile
    fn set_min_frequency(&self, percentage: u8) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                // Update the profile then set it
                cfg.read();
                let profile = cfg.active_profile.clone();
                if let Some(profile) = cfg.power_profiles.get_mut(&profile) {
                    profile.min_percentage = percentage;
                }
                ctrl.set(&profile, &mut cfg)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Modify the active profile
    fn set_max_frequency(&self, percentage: u8) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                // Update the profile then set it
                cfg.read();
                let profile = cfg.active_profile.clone();
                if let Some(profile) = cfg.power_profiles.get_mut(&profile) {
                    profile.max_percentage = percentage;
                }
                ctrl.set(&profile, &mut cfg)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Modify the active profile
    fn set_fan_preset(&self, preset: u8) -> zbus::fdo::Result<()> {
        if preset > 2 {
            return Err(zbus::fdo::Error::InvalidArgs(
                "Fan preset must be 0, 1, or 2".to_string(),
            ));
        }
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                // Update the profile then set it
                cfg.read();
                let profile = cfg.active_profile.clone();
                if let Some(profile) = cfg.power_profiles.get_mut(&profile) {
                    profile.fan_preset = preset;
                }
                ctrl.set(&profile, &mut cfg)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Modify the active profile
    fn set_fan_curve(&self, curve: String) -> zbus::fdo::Result<()> {
        let curve = Curve::from_config_str(&curve)
            .map_err(|err| zbus::fdo::Error::InvalidArgs(format!("Fan curve error: {}", err)))?;
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                // Update the profile then set it
                cfg.read();
                let profile = cfg.active_profile.clone();
                if let Some(profile) = cfg.power_profiles.get_mut(&profile) {
                    profile.fan_curve = Some(curve);
                }
                ctrl.set(&profile, &mut cfg)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Fetch the active profile name
    fn next_profile(&mut self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                cfg.read();
                ctrl.do_next_profile(&mut cfg)
                    .unwrap_or_else(|err| warn!("{}", err));
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string(profile) {
                        self.notify_profile(&json)
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
            }
        }
    }

    /// Fetch the active profile name
    fn active_profile_name(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                return Ok(cfg.active_profile.clone());
            }
        }
        Err(Error::Failed(
            "Failed to get active profile name".to_string(),
        ))
    }

    // TODO: Profile can't implement Type because of Curve
    /// Fetch the active profile details
    fn profile(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string_pretty(profile) {
                        return Ok(json);
                    }
                }
            }
        }
        Err(Error::Failed(
            "Failed to get active profile details".to_string(),
        ))
    }

    /// Fetch all profile data
    fn profiles(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Ok(json) = serde_json::to_string_pretty(&cfg.power_profiles) {
                    return Ok(json);
                }
            }
        }
        Err(Error::Failed(
            "Failed to get all profile details".to_string(),
        ))
    }

    fn profile_names(&self) -> zbus::fdo::Result<Vec<String>> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                let profile_names = cfg.power_profiles.keys().cloned().collect::<Vec<String>>();
                return Ok(profile_names);
            }
        }

        Err(Error::Failed("Failed to get all profile names".to_string()))
    }

    fn remove(&self, profile: &str) -> zbus::fdo::Result<()> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();

                if !cfg.power_profiles.contains_key(profile) {
                    return Err(Error::Failed("Invalid profile specified".to_string()));
                }

                if cfg.power_profiles.keys().len() == 1 {
                    return Err(Error::Failed("Cannot delete the last profile".to_string()));
                }

                if cfg.active_profile == *profile {
                    return Err(Error::Failed(
                        "Cannot delete the active profile".to_string(),
                    ));
                }

                cfg.power_profiles.remove(profile);
                cfg.write();

                return Ok(());
            }
        }

        Err(Error::Failed("Failed to lock configuration".to_string()))
    }

    #[dbus_interface(signal)]
    fn notify_profile(&self, profile: &str) -> zbus::Result<()> {}
}

impl crate::ZbusAdd for FanAndCpuZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Profile"),
                self,
            )
            .map_err(|err| {
                warn!("DbusFanAndCpu: add_to_server {}", err);
                err
            })
            .ok();
    }
}

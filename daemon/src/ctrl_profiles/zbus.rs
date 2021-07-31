use log::warn;
use rog_profiles::profiles::Profile;

use std::sync::Arc;
use std::sync::Mutex;
use zbus::{dbus_interface, fdo::Error};
use zvariant::ObjectPath;

use super::controller::CtrlFanAndCpu;

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
    /// Create new profile and make active
    fn set_profile(&self, profile: String) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.set_active(&profile)
                .unwrap_or_else(|err| warn!("{}", err));
        }
        self.do_notification();
    }

    /// New or modify profile details and make active, will create if it does not exist
    fn new_or_modify(&self, profile: Profile) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.new_or_modify(&profile)
                .unwrap_or_else(|err| warn!("{}", err));
        }
        self.do_notification();
    }

    /// Fetch the active profile name
    fn next_profile(&mut self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.do_next_profile()
                .unwrap_or_else(|err| warn!("{}", err));
        }
        self.do_notification();
    }

    /// Fetch the active profile name
    fn active_name(&mut self) -> zbus::fdo::Result<String> {
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
    fn active_data(&mut self) -> zbus::fdo::Result<Profile> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    return Ok(profile.clone());
                }
            }
        }
        Err(Error::Failed(
            "Failed to get active profile details".to_string(),
        ))
    }

    /// Fetch all profile data
    fn profiles(&mut self) -> zbus::fdo::Result<Vec<Profile>> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                return Ok(cfg.power_profiles.values().cloned().collect());
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
    fn notify_profile(&self, profile: &Profile) -> zbus::Result<()> {}
}

impl FanAndCpuZbus {
    fn do_notification(&self) {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(cfg) = ctrl.config.clone().try_lock() {
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    self.notify_profile(profile)
                        .unwrap_or_else(|err| warn!("{}", err));
                }
            }
        }
    }
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

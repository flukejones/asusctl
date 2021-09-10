use log::warn;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::fan_curve_set::FanCurveSet;
use rog_profiles::Profile;

use std::sync::Arc;
use std::sync::Mutex;
use zbus::{dbus_interface, fdo::Error};
use zvariant::ObjectPath;

use super::controller::CtrlPlatformProfile;

static UNSUPPORTED_MSG: &str =
    "Fan curves are not supported on this laptop or you require a patched kernel";

pub struct ProfileZbus {
    inner: Arc<Mutex<CtrlPlatformProfile>>,
}

impl ProfileZbus {
    pub fn new(inner: Arc<Mutex<CtrlPlatformProfile>>) -> Self {
        Self { inner }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl ProfileZbus {
    /// Fetch profile names
    fn profiles(&mut self) -> zbus::fdo::Result<Vec<Profile>> {
        if let Ok(profiles) = Profile::get_profile_names() {
            return Ok(profiles);
        }
        Err(Error::Failed(
            "Failed to get all profile details".to_string(),
        ))
    }

    /// Toggle to next platform_profile. Names provided by `Profiles`
    fn next_profile(&mut self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.set_next_profile()
                .unwrap_or_else(|err| warn!("{}", err));
        }
        self.do_notification();
    }

    /// Fetch the active profile name
    fn active_profile(&mut self) -> zbus::fdo::Result<Profile> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            return Ok(ctrl.config.active_profile);
        }
        Err(Error::Failed(
            "Failed to get active profile name".to_string(),
        ))
    }

    /// Set this platform_profile name as active
    fn set_active_profile(&self, profile: Profile) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            // Read first just incase the user has modified the config before calling this
            ctrl.config.read();
            Profile::set_profile(profile)
                .map_err(|e| warn!("Profile::set_profile, {}", e))
                .ok();
            ctrl.config.active_profile = profile;

            ctrl.save_config();
        }
        self.do_notification();
    }

    /// Get a list of profiles that have fan-curves enabled.
    fn enabled_fan_profiles(&mut self) -> zbus::fdo::Result<Vec<Profile>> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            if let Some(curves) = &ctrl.config.fan_curves {
                return Ok(curves.get_enabled_curve_profiles().to_vec());
            }
            return Err(Error::Failed(UNSUPPORTED_MSG.to_string()));
        }
        Err(Error::Failed(
            "Failed to get enabled fan curve names".to_string(),
        ))
    }

    /// Get a list of profiles that have fan-curves enabled.
    fn set_enabled_fan_profiles(&mut self, profiles: Vec<Profile>) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            if let Some(curves) = &mut ctrl.config.fan_curves {
                curves.set_enabled_curve_profiles(profiles);
            }
            return Err(Error::Failed(UNSUPPORTED_MSG.to_string()));
        }
        Err(Error::Failed(
            "Failed to get enabled fan curve names".to_string(),
        ))
    }

    /// Get the fan-curve data for the currently active Profile
    fn active_fan_curve_data(&mut self) -> zbus::fdo::Result<FanCurveSet> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            if let Some(curves) = &ctrl.config.fan_curves {
                return Ok((*curves.get_active_fan_curves()).clone());
            }
            return Err(Error::Failed(UNSUPPORTED_MSG.to_string()));
        }
        Err(Error::Failed("Failed to get fan curve data".to_string()))
    }

    /// Get fan-curve data for each Profile as an array of objects
    fn fan_curves(&self) -> zbus::fdo::Result<Vec<FanCurveSet>> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            if let Some(curves) = &ctrl.config.fan_curves {
                return Ok(curves.get_all_fan_curves());
            }
            return Err(Error::Failed(UNSUPPORTED_MSG.to_string()));
        }
        Err(Error::Failed("Failed to get all fan curves".to_string()))
    }

    /// Set this fan-curve data
    fn set_fan_curve(&self, curve: CurveData) -> zbus::fdo::Result<()> {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.config.read();
            let profile = ctrl.config.active_profile;
            if let Some(mut device) = ctrl.get_device() {
                if let Some(curves) = &mut ctrl.config.fan_curves {
                    curves.write_and_set_fan_curve(curve, profile, &mut device);
                }
            } else {
                return Err(Error::Failed(UNSUPPORTED_MSG.to_string()));
            }

            ctrl.save_config();
        }

        Err(Error::Failed("Failed to set fan curves".to_string()))
    }

    #[dbus_interface(signal)]
    fn notify_profile(&self, profile: &Profile) -> zbus::Result<()> {}
}

impl ProfileZbus {
    fn do_notification(&self) {
        if let Ok(ctrl) = self.inner.try_lock() {
            self.notify_profile(&ctrl.config.active_profile)
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }
}

impl crate::ZbusAdd for ProfileZbus {
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

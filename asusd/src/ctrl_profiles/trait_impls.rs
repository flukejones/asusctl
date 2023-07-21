use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::StdConfig;
use log::{error, info, warn};
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::{FanCurveProfiles, Profile};
use zbus::export::futures_util::lock::Mutex;
use zbus::export::futures_util::StreamExt;
use zbus::fdo::Error;
use zbus::{dbus_interface, Connection, SignalContext};

use super::controller::CtrlPlatformProfile;
use crate::error::RogError;
use crate::CtrlTask;

const MOD_NAME: &str = "ProfileZbus";

const ZBUS_PATH: &str = "/org/asuslinux/Profile";
const UNSUPPORTED_MSG: &str =
    "Fan curves are not supported on this laptop or you require a patched kernel";

#[derive(Clone)]
pub struct ProfileZbus(pub Arc<Mutex<CtrlPlatformProfile>>);

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl ProfileZbus {
    /// Fetch profile names
    fn profiles(&mut self) -> zbus::fdo::Result<Vec<Profile>> {
        if let Ok(profiles) = Profile::get_profile_names() {
            return Ok(profiles);
        }
        Err(Error::Failed(
            "Failed to get all profile details".to_owned(),
        ))
    }

    /// Toggle to next platform_profile. Names provided by `Profiles`.
    /// If fan-curves are supported will also activate a fan curve for profile.
    async fn next_profile(&mut self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
        let mut ctrl = self.0.lock().await;
        let next = Profile::get_next_profile(ctrl.profile_config.active_profile);
        Profile::set_profile(next)
            .map_err(|e| warn!("{MOD_NAME}: set_profile, {}", e))
            .ok();
        ctrl.profile_config.active_profile = next;
        ctrl.save_config();

        Self::notify_profile(&ctxt, ctrl.profile_config.active_profile)
            .await
            .ok();
    }

    /// Fetch the active profile name
    async fn active_profile(&mut self) -> zbus::fdo::Result<Profile> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        Ok(ctrl.profile_config.active_profile)
    }

    /// Set this platform_profile name as active
    async fn set_active_profile(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        profile: Profile,
    ) {
        let mut ctrl = self.0.lock().await;
        // Read first just incase the user has modified the config before calling this
        ctrl.profile_config.read();
        Profile::set_profile(profile)
            .map_err(|e| warn!("{MOD_NAME}: set_profile, {}", e))
            .ok();
        ctrl.profile_config.active_profile = profile;
        ctrl.write_profile_curve_to_platform()
            .map_err(|e| warn!("{MOD_NAME}: write_profile_curve_to_platform, {}", e))
            .ok();

        ctrl.save_config();

        Self::notify_profile(&ctxt, ctrl.profile_config.active_profile)
            .await
            .ok();
    }

    /// Set a profile fan curve enabled status. Will also activate a fan curve
    /// if in the same profile mode
    async fn set_fan_curve_enabled(
        &mut self,
        profile: Profile,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        if let Some(curves) = &mut ctrl.fan_curves {
            curves
                .profiles_mut()
                .set_profile_curve_enabled(profile, enabled);

            ctrl.write_profile_curve_to_platform()
                .map_err(|e| warn!("{MOD_NAME}: write_profile_curve_to_platform, {}", e))
                .ok();

            ctrl.save_config();
            Ok(())
        } else {
            Err(Error::Failed(UNSUPPORTED_MSG.to_owned()))
        }
    }

    /// Get the fan-curve data for the currently active Profile
    async fn fan_curve_data(&mut self, profile: Profile) -> zbus::fdo::Result<Vec<CurveData>> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        if let Some(curves) = &mut ctrl.fan_curves {
            let curve = curves.profiles().get_fan_curves_for(profile);
            return Ok(curve.to_vec());
        }
        Err(Error::Failed(UNSUPPORTED_MSG.to_owned()))
    }

    /// Set the fan curve for the specified profile.
    /// Will also activate the fan curve if the user is in the same mode.
    async fn set_fan_curve(&self, profile: Profile, curve: CurveData) -> zbus::fdo::Result<()> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        if let Some(curves) = &mut ctrl.fan_curves {
            curves
                .profiles_mut()
                .save_fan_curve(curve, profile)
                .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
        } else {
            return Err(Error::Failed(UNSUPPORTED_MSG.to_owned()));
        }
        ctrl.write_profile_curve_to_platform()
            .map_err(|e| warn!("{MOD_NAME}: Profile::set_profile, {}", e))
            .ok();
        ctrl.save_config();

        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    async fn set_active_curve_to_defaults(&self) -> zbus::fdo::Result<()> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        ctrl.set_active_curve_to_defaults()
            .map_err(|e| warn!("{MOD_NAME}: Profile::set_active_curve_to_defaults, {}", e))
            .ok();
        ctrl.save_config();
        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    async fn reset_profile_curves(&self, profile: Profile) -> zbus::fdo::Result<()> {
        let mut ctrl = self.0.lock().await;
        ctrl.profile_config.read();
        let active = Profile::get_active_profile().unwrap_or(Profile::Balanced);

        Profile::set_profile(profile)
            .map_err(|e| warn!("{MOD_NAME}: set_profile, {}", e))
            .ok();
        ctrl.set_active_curve_to_defaults()
            .map_err(|e| warn!("{MOD_NAME}: Profile::set_active_curve_to_defaults, {}", e))
            .ok();

        Profile::set_profile(active)
            .map_err(|e| warn!("{MOD_NAME}: set_profile, {}", e))
            .ok();
        ctrl.save_config();
        Ok(())
    }

    #[dbus_interface(signal)]
    async fn notify_profile(signal_ctxt: &SignalContext<'_>, profile: Profile) -> zbus::Result<()> {
    }
}

#[async_trait]
impl crate::ZbusRun for ProfileZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, ZBUS_PATH, server).await;
    }
}

#[async_trait]
impl CtrlTask for ProfileZbus {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let ctrl = self.0.clone();
        let sig_ctx = signal_ctxt.clone();
        let watch = self
            .0
            .lock()
            .await
            .platform
            .monitor_throttle_thermal_policy()?;

        tokio::spawn(async move {
            let mut buffer = [0; 32];
            if let Ok(stream) = watch.into_event_stream(&mut buffer) {
                stream
                    .for_each(|_| async {
                        let mut lock = ctrl.lock().await;
                        if let Ok(profile) =
                            lock.platform.get_throttle_thermal_policy().map_err(|e| {
                                error!("{MOD_NAME}: get_throttle_thermal_policy error: {e}");
                            })
                        {
                            let new_profile = Profile::from_throttle_thermal_policy(profile);
                            if new_profile != lock.profile_config.active_profile {
                                info!("{MOD_NAME}: platform_profile changed to {new_profile}");
                                lock.profile_config.active_profile = new_profile;
                                lock.write_profile_curve_to_platform().unwrap();
                                lock.save_config();
                                Profile::set_profile(lock.profile_config.active_profile)
                                    .map_err(|e| {
                                        error!("Profile::set_profile() error: {e}");
                                    })
                                    .ok();

                                Self::notify_profile(&sig_ctx, lock.profile_config.active_profile)
                                    .await
                                    .ok();
                            }
                        }
                    })
                    .await;
            }
        });

        let ctrl = self.0.clone();
        let watch = self.0.lock().await.platform.monitor_platform_profile()?;

        tokio::spawn(async move {
            let mut buffer = [0; 32];
            if let Ok(stream) = watch.into_event_stream(&mut buffer) {
                stream
                    .for_each(|_| async {
                        let mut lock = ctrl.lock().await;
                        if let Ok(profile) = lock.platform.get_platform_profile().map_err(|e| {
                            error!("get_platform_profile error: {e}");
                        }) {
                            if let Ok(new_profile) = Profile::from_str(&profile).map_err(|e| {
                                error!("Profile::from_str(&profile) error: {e}");
                            }) {
                                if new_profile != lock.profile_config.active_profile {
                                    info!("{MOD_NAME}: platform_profile changed to {new_profile}");
                                    lock.profile_config.active_profile = new_profile;
                                    lock.write_profile_curve_to_platform().unwrap();
                                    lock.save_config();
                                    Profile::set_profile(lock.profile_config.active_profile)
                                        .map_err(|e| {
                                            error!("Profile::set_profile() error: {e}");
                                        })
                                        .ok();

                                    Self::notify_profile(
                                        &signal_ctxt,
                                        lock.profile_config.active_profile,
                                    )
                                    .await
                                    .ok();
                                }
                            }
                        }
                    })
                    .await;
            }
        });

        Ok(())
    }
}

#[async_trait]
impl crate::Reloadable for ProfileZbus {
    /// Fetch the active profile and use that to set all related components up
    async fn reload(&mut self) -> Result<(), RogError> {
        let mut ctrl = self.0.lock().await;
        let active = ctrl.profile_config.active_profile;
        if let Some(curves) = &mut ctrl.fan_curves {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                // There is a possibility that the curve was default zeroed, so this call
                // initialises the data from system read and we need to save it
                // after
                curves
                    .profiles_mut()
                    .write_profile_curve_to_platform(active, &mut device)?;
                ctrl.profile_config.write();
            }
        }
        Ok(())
    }
}

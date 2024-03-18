use std::path::PathBuf;
use std::sync::Arc;

use config_traits::{StdConfig, StdConfigLoad};
use futures_lite::StreamExt;
use log::{debug, error, info, warn};
use rog_platform::platform::{RogPlatform, ThrottlePolicy};
use rog_profiles::error::ProfileError;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::{find_fan_curve_node, FanCurvePU, FanCurveProfiles};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::Mutex;
use zbus::{interface, Connection, SignalContext};

use crate::error::RogError;
use crate::{CtrlTask, CONFIG_PATH_BASE};

pub const FAN_CURVE_ZBUS_NAME: &str = "FanCurves";
pub const FAN_CURVE_ZBUS_PATH: &str = "/org/asuslinux";

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveConfig {
    pub profiles: FanCurveProfiles,
    #[serde(skip)]
    pub current: u8,
}

impl StdConfig for FanCurveConfig {
    /// Create a new config. The defaults are zeroed so the device must be read
    /// to get the actual device defaults.
    fn new() -> Self {
        Self::default()
    }

    fn file_name(&self) -> String {
        "fan_curves.ron".to_owned()
    }

    fn config_dir() -> std::path::PathBuf {
        PathBuf::from(CONFIG_PATH_BASE)
    }
}

impl StdConfigLoad for FanCurveConfig {}

#[derive(Debug, Clone)]
pub struct CtrlFanCurveZbus {
    config: Arc<Mutex<FanCurveConfig>>,
    platform: RogPlatform,
}

// Non-zbus-derive impl
impl CtrlFanCurveZbus {
    pub fn new() -> Result<Self, RogError> {
        let platform = RogPlatform::new()?;
        if platform.has_throttle_thermal_policy() {
            info!("Device has profile control available");
            find_fan_curve_node()?;
            info!("Device has fan curves available");
            let mut config = FanCurveConfig::new().load();
            let mut fan_curves = FanCurveProfiles::default();

            // Only do defaults if the config doesn't already exist\
            if config.profiles.balanced.is_empty() || !config.file_path().exists() {
                info!("Fetching default fan curves");

                let current = platform.get_throttle_thermal_policy()?;
                for this in [
                    ThrottlePolicy::Balanced,
                    ThrottlePolicy::Performance,
                    ThrottlePolicy::Quiet,
                ] {
                    // For each profile we need to switch to it before we
                    // can read the existing values from hardware. The ACPI method used
                    // for this is what limits us.
                    platform.set_throttle_thermal_policy(this.into())?;
                    let mut dev = find_fan_curve_node()?;
                    fan_curves.set_active_curve_to_defaults(this, &mut dev)?;

                    info!("{this:?}:");
                    for curve in fan_curves.get_fan_curves_for(this) {
                        info!("{}", String::from(curve));
                    }
                }
                platform.set_throttle_thermal_policy(current)?;
                config.profiles = fan_curves;
                config.write();
            } else {
                info!("Fan curves previously stored, loading...");
                config = config.load();
            }

            return Ok(Self {
                config: Arc::new(Mutex::new(config)),
                platform,
            });
        }

        Err(ProfileError::NotSupported.into())
    }
}

#[interface(name = "org.asuslinux.FanCurves")]
impl CtrlFanCurveZbus {
    /// Set all fan curves for a profile to enabled status. Will also activate a
    /// fan curve if in the same profile mode
    async fn set_fan_curves_enabled(
        &mut self,
        profile: ThrottlePolicy,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        self.config
            .lock()
            .await
            .profiles
            .set_profile_curves_enabled(profile, enabled);
        self.config
            .lock()
            .await
            .profiles
            .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        self.config.lock().await.write();
        Ok(())
    }

    /// Set a single fan curve for a profile to enabled status. Will also
    /// activate a fan curve if in the same profile mode
    async fn set_profile_fan_curve_enabled(
        &mut self,
        profile: ThrottlePolicy,
        fan: FanCurvePU,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        self.config
            .lock()
            .await
            .profiles
            .set_profile_fan_curve_enabled(profile, fan, enabled);
        self.config
            .lock()
            .await
            .profiles
            .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        self.config.lock().await.write();
        Ok(())
    }

    /// Get the fan-curve data for the currently active ThrottlePolicy
    async fn fan_curve_data(
        &mut self,
        profile: ThrottlePolicy,
    ) -> zbus::fdo::Result<Vec<CurveData>> {
        let curve = self
            .config
            .lock()
            .await
            .profiles
            .get_fan_curves_for(profile)
            .to_vec();
        Ok(curve)
    }

    /// Set the fan curve for the specified profile.
    /// Will also activate the fan curve if the user is in the same mode.
    async fn set_fan_curve(
        &mut self,
        profile: ThrottlePolicy,
        curve: CurveData,
    ) -> zbus::fdo::Result<()> {
        self.config
            .lock()
            .await
            .profiles
            .save_fan_curve(curve, profile)?;
        let active: ThrottlePolicy = self.platform.get_throttle_thermal_policy()?.into();
        if active == profile {
            self.config
                .lock()
                .await
                .profiles
                .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        }
        self.config.lock().await.write();
        Ok(())
    }

    /// Reset the stored (self) and device curves to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the default can be
    /// read only for the currently active profile.
    async fn set_curves_to_defaults(&mut self, profile: ThrottlePolicy) -> zbus::fdo::Result<()> {
        let active = self.platform.get_throttle_thermal_policy()?;
        self.platform.set_throttle_thermal_policy(profile.into())?;
        self.config
            .lock()
            .await
            .profiles
            .set_active_curve_to_defaults(profile, &mut find_fan_curve_node()?)?;
        self.platform.set_throttle_thermal_policy(active)?;
        self.config.lock().await.write();
        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    async fn reset_profile_curves(&self, profile: ThrottlePolicy) -> zbus::fdo::Result<()> {
        let active = self.platform.get_throttle_thermal_policy()?;

        self.platform.set_throttle_thermal_policy(profile.into())?;
        self.config
            .lock()
            .await
            .profiles
            .set_active_curve_to_defaults(active.into(), &mut find_fan_curve_node()?)?;
        self.platform.set_throttle_thermal_policy(active)?;

        self.config.lock().await.write();
        Ok(())
    }
}

impl crate::ZbusRun for CtrlFanCurveZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, FAN_CURVE_ZBUS_PATH, server).await;
    }
}

impl CtrlTask for CtrlFanCurveZbus {
    fn zbus_path() -> &'static str {
        FAN_CURVE_ZBUS_PATH
    }

    async fn create_tasks(&self, _signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let watch_throttle_thermal_policy = self.platform.monitor_throttle_thermal_policy()?;
        let platform = self.platform.clone();
        let config = self.config.clone();
        let fan_curves = self.config.clone();

        tokio::spawn(async move {
            let mut buffer = [0; 32];
            if let Ok(mut stream) = watch_throttle_thermal_policy.into_event_stream(&mut buffer) {
                while (stream.next().await).is_some() {
                    debug!("watch_throttle_thermal_policy changed");
                    if let Ok(profile) = platform.get_throttle_thermal_policy().map_err(|e| {
                        error!("get_throttle_thermal_policy error: {e}");
                    }) {
                        if profile != config.lock().await.current {
                            fan_curves
                                .lock()
                                .await
                                .profiles
                                .write_profile_curve_to_platform(
                                    profile.into(),
                                    &mut find_fan_curve_node().unwrap(),
                                )
                                .map_err(|e| warn!("write_profile_curve_to_platform, {}", e))
                                .ok();
                            config.lock().await.current = profile;
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

impl crate::Reloadable for CtrlFanCurveZbus {
    /// Fetch the active profile and use that to set all related components up
    async fn reload(&mut self) -> Result<(), RogError> {
        let active = self.platform.get_throttle_thermal_policy()?.into();
        if let Ok(mut device) = find_fan_curve_node() {
            // There is a possibility that the curve was default zeroed, so this call
            // initialises the data from system read and we need to save it
            // after
            loop {
                if let Ok(mut config) = self.config.try_lock() {
                    config
                        .profiles
                        .write_profile_curve_to_platform(active, &mut device)?;
                    break;
                }
            }
        }
        Ok(())
    }
}

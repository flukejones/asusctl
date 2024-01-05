use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use config_traits::{StdConfig, StdConfigLoad};
use futures_lite::StreamExt;
use log::{debug, error, info, warn};
use rog_platform::platform::{PlatformPolicy, RogPlatform};
use rog_profiles::error::ProfileError;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::{find_fan_curve_node, FanCurvePU, FanCurveProfiles};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::Mutex;
use zbus::{dbus_interface, Connection, SignalContext};

use crate::error::RogError;
use crate::{CtrlTask, CONFIG_PATH_BASE};

const MOD_NAME: &str = "FanCurveZbus";
pub const FAN_CURVE_ZBUS_NAME: &str = "FanCurves";
pub const FAN_CURVE_ZBUS_PATH: &str = "/org/asuslinux/FanCurves";

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveConfig {
    pub balanced: Vec<CurveData>,
    pub performance: Vec<CurveData>,
    pub quiet: Vec<CurveData>,
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
    fan_curves: Arc<Mutex<FanCurveProfiles>>,
    platform: RogPlatform,
}

// Non-zbus-derive impl
impl CtrlFanCurveZbus {
    pub fn new() -> Result<Self, RogError> {
        let platform = RogPlatform::new()?;
        if platform.has_throttle_thermal_policy() {
            info!("{MOD_NAME}: Device has profile control available");
            find_fan_curve_node()?;
            info!("{MOD_NAME}: Device has fan curves available");
            let mut config = FanCurveConfig::new();
            let mut fan_curves = FanCurveProfiles::default();

            // Only do defaults if the config doesn't already exist
            if !config.file_path().exists() {
                info!("{MOD_NAME}: Fetching default fan curves");

                for this in [
                    PlatformPolicy::Balanced,
                    PlatformPolicy::Performance,
                    PlatformPolicy::Quiet,
                ] {
                    // For each profile we need to switch to it before we
                    // can read the existing values from hardware. The ACPI method used
                    // for this is what limits us.
                    let next = PlatformPolicy::get_next_profile(this);
                    platform.set_throttle_thermal_policy(next.into())?;

                    let active = platform
                        .get_throttle_thermal_policy()
                        .map_or(PlatformPolicy::Balanced, |t| t.into());

                    info!("{MOD_NAME}: {active:?}:");
                    for curve in fan_curves.get_fan_curves_for(active) {
                        info!("{}", String::from(curve));
                    }
                }
                config.write();
            } else {
                info!("{MOD_NAME}: Fan curves previously stored, loading...");
                config = config.load();
                fan_curves.balanced = config.balanced.clone();
                fan_curves.performance = config.performance.clone();
                fan_curves.quiet = config.quiet.clone();
            }

            return Ok(Self {
                config: Arc::new(Mutex::new(config)),
                fan_curves: Arc::new(Mutex::new(fan_curves)),
                platform,
            });
        }

        Err(ProfileError::NotSupported.into())
    }

    pub async fn update_profiles_from_config(&self) {
        self.fan_curves.lock().await.balanced = self.config.lock().await.balanced.clone();
        self.fan_curves.lock().await.performance = self.config.lock().await.performance.clone();
        self.fan_curves.lock().await.quiet = self.config.lock().await.quiet.clone();
    }

    /// Because this locks both config and fan_curves, it means nothing else can
    /// hold a lock across this function call. Stupid choice to do this and
    /// needs to be fixed.
    pub async fn update_config_from_profiles(&self) {
        self.config.lock().await.balanced = self.fan_curves.lock().await.balanced.clone();
        self.config.lock().await.performance = self.fan_curves.lock().await.performance.clone();
        self.config.lock().await.quiet = self.fan_curves.lock().await.quiet.clone();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlFanCurveZbus {
    /// Set all fan curves for a profile to enabled status. Will also activate a
    /// fan curve if in the same profile mode
    async fn set_fan_curves_enabled(
        &mut self,
        profile: PlatformPolicy,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        self.fan_curves
            .lock()
            .await
            .set_profile_curves_enabled(profile, enabled);
        self.fan_curves
            .lock()
            .await
            .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        self.update_config_from_profiles().await;
        self.config.lock().await.write();
        Ok(())
    }

    /// Set a single fan curve for a profile to enabled status. Will also
    /// activate a fan curve if in the same profile mode
    async fn set_profile_fan_curve_enabled(
        &mut self,
        profile: PlatformPolicy,
        fan: FanCurvePU,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        self.fan_curves
            .lock()
            .await
            .set_profile_fan_curve_enabled(profile, fan, enabled);
        self.fan_curves
            .lock()
            .await
            .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        self.update_config_from_profiles().await;
        self.config.lock().await.write();
        Ok(())
    }

    /// Get the fan-curve data for the currently active PlatformPolicy
    async fn fan_curve_data(
        &mut self,
        profile: PlatformPolicy,
    ) -> zbus::fdo::Result<Vec<CurveData>> {
        let curve = self
            .fan_curves
            .lock()
            .await
            .get_fan_curves_for(profile)
            .to_vec();
        Ok(curve)
    }

    /// Set the fan curve for the specified profile.
    /// Will also activate the fan curve if the user is in the same mode.
    async fn set_fan_curve(
        &mut self,
        profile: PlatformPolicy,
        curve: CurveData,
    ) -> zbus::fdo::Result<()> {
        self.fan_curves
            .lock()
            .await
            .save_fan_curve(curve, profile)?;
        self.fan_curves
            .lock()
            .await
            .write_profile_curve_to_platform(profile, &mut find_fan_curve_node()?)?;
        self.update_config_from_profiles().await;
        self.config.lock().await.write();
        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    async fn set_active_curve_to_defaults(&mut self) -> zbus::fdo::Result<()> {
        let active = self.platform.get_throttle_thermal_policy()?;
        self.fan_curves
            .lock()
            .await
            .set_active_curve_to_defaults(active.into(), &mut find_fan_curve_node()?)?;
        self.update_config_from_profiles().await;
        self.config.lock().await.write();
        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    async fn reset_profile_curves(&self, profile: PlatformPolicy) -> zbus::fdo::Result<()> {
        let active = self
            .platform
            .get_throttle_thermal_policy()
            .unwrap_or(PlatformPolicy::Balanced.into());

        self.platform.set_throttle_thermal_policy(profile.into())?;
        self.fan_curves
            .lock()
            .await
            .set_active_curve_to_defaults(active.into(), &mut find_fan_curve_node()?)?;
        self.platform.set_throttle_thermal_policy(active)?;

        self.update_config_from_profiles().await;
        self.config.lock().await.write();
        Ok(())
    }
}

#[async_trait]
impl crate::ZbusRun for CtrlFanCurveZbus {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, FAN_CURVE_ZBUS_PATH, server).await;
    }
}

#[async_trait]
impl CtrlTask for CtrlFanCurveZbus {
    fn zbus_path() -> &'static str {
        FAN_CURVE_ZBUS_PATH
    }

    async fn create_tasks(&self, _signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let watch_throttle_thermal_policy = self.platform.monitor_throttle_thermal_policy()?;
        let platform = self.platform.clone();
        let config = self.config.clone();
        let fan_curves = self.fan_curves.clone();

        tokio::spawn(async move {
            let mut buffer = [0; 32];
            if let Ok(mut stream) = watch_throttle_thermal_policy.into_event_stream(&mut buffer) {
                while (stream.next().await).is_some() {
                    debug!("watch_throttle_thermal_policy changed");
                    if let Ok(profile) = platform.get_throttle_thermal_policy().map_err(|e| {
                        error!("{MOD_NAME}: get_throttle_thermal_policy error: {e}");
                    }) {
                        if profile != config.lock().await.current {
                            fan_curves
                                .lock()
                                .await
                                .write_profile_curve_to_platform(
                                    profile.into(),
                                    &mut find_fan_curve_node().unwrap(),
                                )
                                .map_err(|e| {
                                    warn!("{MOD_NAME}: write_profile_curve_to_platform, {}", e)
                                })
                                .ok();
                            config.lock().await.current = profile;
                        }
                    }
                }
                dbg!("STREAM ENDED");
            }
        });

        Ok(())
    }
}

#[async_trait]
impl crate::Reloadable for CtrlFanCurveZbus {
    /// Fetch the active profile and use that to set all related components up
    async fn reload(&mut self) -> Result<(), RogError> {
        let active = self.platform.get_throttle_thermal_policy()?.into();
        if let Ok(mut device) = find_fan_curve_node() {
            // There is a possibility that the curve was default zeroed, so this call
            // initialises the data from system read and we need to save it
            // after
            loop {
                if let Ok(mut curves) = self.fan_curves.try_lock() {
                    curves.write_profile_curve_to_platform(active, &mut device)?;
                    break;
                }
            }
        }
        Ok(())
    }
}

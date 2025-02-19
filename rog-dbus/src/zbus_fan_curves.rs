//! # `DBus` interface proxy for: `xyz.ljones.Asusd`
//!
//! This code was generated by `zbus-xmlgen` `1.0.0` from `DBus` introspection
//! data. Source: `Interface '/xyz/ljones/Profile' from service
//! 'xyz.ljones.Asusd' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the
//! [Writing a client proxy](https://zeenix.pages.freedesktop.org/zbus/client.html)
//! section of the zbus documentation.
//!
//! This `DBus object implements
//! [standard DBus` interfaces](https://dbus.freedesktop.org/doc/dbus-specification.html),
//! (`org.freedesktop.DBus.*`) for which the following zbus proxies can be used:
//!
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//! * [`zbus::fdo::PropertiesProxy`]
//!
//! …consequently `zbus-xmlgen` did not generate code for the above interfaces.

use rog_platform::platform::PlatformProfile;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::FanCurvePU;
use zbus::proxy;

#[proxy(
    interface = "xyz.ljones.FanCurves",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones"
)]
pub trait FanCurves {
    /// Get the fan-curve data for the currently active PlatformProfile
    fn fan_curve_data(&self, profile: PlatformProfile) -> zbus::Result<Vec<CurveData>>;

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each platform_profile has a different default and the defualt can be
    /// read only for the currently active profile.
    fn reset_profile_curves(&self, profile: PlatformProfile) -> zbus::Result<()>;

    /// SetActiveCurveToDefaults method
    fn set_curves_to_defaults(&self, profile: PlatformProfile) -> zbus::Result<()>;

    /// Set the fan curve for the specified profile, or the profile the user is
    /// currently in if profile == None. Will also activate the fan curve.
    fn set_fan_curve(&self, profile: PlatformProfile, curve: CurveData) -> zbus::Result<()>;

    /// Set a profile fan curve enabled status. Will also activate a fan curve.
    fn set_fan_curves_enabled(&self, profile: PlatformProfile, enabled: bool) -> zbus::Result<()>;

    /// Set a single fan curve for a profile to enabled status. Will also
    /// activate a fan curve.
    async fn set_profile_fan_curve_enabled(
        &self,
        profile: PlatformProfile,
        fan: FanCurvePU,
        enabled: bool,
    ) -> zbus::Result<()>;
}

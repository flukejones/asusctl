//! # D-Bus interface proxy for: `xyz.ljones.AsusArmoury`
//!
//! `zbus-xmlgen system xyz.ljones.Asusd
//! /xyz/ljones/asus_armoury/nv_temp_target`
use zbus::proxy;
#[proxy(
    interface = "xyz.ljones.AsusArmoury",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones/asus_armoury/nv_temp_target"
)]
pub trait AsusArmoury {
    /// A list of the properties this attribute actually uses. Any property
    /// not listed will return either an empty array or `-1`
    #[zbus(property)]
    fn available_attrs(&self) -> zbus::Result<Vec<String>>;

    /// CurrentValue property
    #[zbus(property)]
    fn current_value(&self) -> zbus::Result<i32>;
    #[zbus(property)]
    fn set_current_value(&self, value: i32) -> zbus::Result<()>;

    /// DefaultValue property
    #[zbus(property)]
    fn default_value(&self) -> zbus::Result<i32>;

    /// MaxValue property. Maximum allowed current_value. Returns `-1` if unused
    /// or not set.
    #[zbus(property)]
    fn max_value(&self) -> zbus::Result<i32>;

    /// MinValue property. Minimum allowed current_value. Returns `-1` if unused
    /// or not set.
    #[zbus(property)]
    fn min_value(&self) -> zbus::Result<i32>;

    /// PossibleValues property. Return the allowed values for `current_value`
    /// if used or set, otherwise the array is empty.
    #[zbus(property)]
    fn possible_values(&self) -> zbus::Result<Vec<i32>>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// ScalarIncrement property. The increment steps that `current_value` may
    /// take. Returns `-1` if not used or set.
    #[zbus(property)]
    fn scalar_increment(&self) -> zbus::Result<i32>;
}

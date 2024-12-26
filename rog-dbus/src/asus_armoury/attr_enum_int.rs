//! Path for iface is such as "/xyz/ljones/asus_armoury/boot_sound"
use zbus::proxy;
#[proxy(
    interface = "xyz.ljones.AsusArmoury",
    default_service = "xyz.ljones.Asusd"
)]
pub trait AsusArmoury {
    /// CurrentValue property
    #[zbus(property)]
    fn current_value(&self) -> zbus::Result<i32>;
    #[zbus(property)]
    fn set_current_value(&self, value: i32) -> zbus::Result<()>;

    /// DefaultValue property
    #[zbus(property)]
    fn default_value(&self) -> zbus::Result<i32>;

    /// MaxValue property
    #[zbus(property)]
    fn max_value(&self) -> zbus::Result<i32>;

    /// MinValue property
    #[zbus(property)]
    fn min_value(&self) -> zbus::Result<i32>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// ScalarIncrement property
    #[zbus(property)]
    fn scalar_increment(&self) -> zbus::Result<i32>;
}

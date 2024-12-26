//! Path for iface is such as "/xyz/ljones/asus_armoury/boot_sound"
use zbus::proxy;
#[proxy(
    interface = "xyz.ljones.AsusArmoury",
    default_service = "xyz.ljones.Asusd"
)]
pub trait AsusArmoury {
    /// CurrentValue property
    #[zbus(property)]
    fn current_value(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_current_value(&self, value: String) -> zbus::Result<()>;

    /// DefaultValue property
    #[zbus(property)]
    fn default_value(&self) -> zbus::Result<String>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// PossibleValues property
    #[zbus(property)]
    fn possible_values(&self) -> zbus::Result<Vec<String>>;
}

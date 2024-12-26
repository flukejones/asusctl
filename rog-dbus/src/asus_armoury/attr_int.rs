use zbus::proxy;
#[proxy(
    interface = "xyz.ljones.AsusArmoury",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones/asus_armoury/boot_sound"
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

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// PossibleValues property
    #[zbus(property)]
    fn possible_values(&self) -> zbus::Result<Vec<i32>>;
}

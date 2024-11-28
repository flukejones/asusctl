use rog_slash::SlashMode;
use zbus::proxy;

#[proxy(
    interface = "org.asuslinux.Slash",
    default_service = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux"
)]
pub trait Slash {
    /// EnableDisplay property
    #[zbus(property)]
    fn enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_enabled(&self, value: bool) -> zbus::Result<()>;

    /// Brightness property
    #[zbus(property)]
    fn brightness(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_brightness(&self, value: u8) -> zbus::Result<()>;

    /// Interval property
    #[zbus(property)]
    fn interval(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_interval(&self, value: u8) -> zbus::Result<()>;

    /// Slash modes property
    #[zbus(property)]
    fn slash_mode(&self) -> zbus::Result<SlashMode>;
    #[zbus(property)]
    fn set_slash_mode(&self, value: SlashMode) -> zbus::Result<()>;
}

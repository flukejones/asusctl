use zbus::proxy;

#[proxy(
interface = "org.asuslinux.Slash",
default_service = "org.asuslinux.Daemon",
default_path = "/org/asuslinux"
)]
trait Slash {
    /// RunMainLoop method
    fn run_main_loop(&self, start: bool) -> zbus::Result<()>;

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

    /// BuiltinAnimations property
    #[zbus(property)]
    fn current_mode(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_current_mode(&self, value: u8) -> zbus::Result<()>;

    /// EnableDisplay property
    #[zbus(property)]
    fn enable_display(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_enable_display(&self, value: bool) -> zbus::Result<()>;
}

use rog_anime::usb::Brightness;
use rog_anime::{Animations, AnimeDataBuffer, DeviceState as AnimeDeviceState};
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.asuslinux.Daemon",
    default_service = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux/Anime"
)]
trait Anime {
    /// DeviceState method
    fn device_state(&self) -> zbus::Result<AnimeDeviceState>;

    /// RunMainLoop method
    fn run_main_loop(&self, start: bool) -> zbus::Result<()>;

    /// Write method
    fn write(&self, input: AnimeDataBuffer) -> zbus::Result<()>;

    /// NotifyDeviceState signal
    #[dbus_proxy(signal)]
    fn notify_device_state(&self, data: AnimeDeviceState) -> zbus::Result<()>;

    /// Brightness property
    #[dbus_proxy(property)]
    fn brightness(&self) -> zbus::Result<Brightness>;
    #[dbus_proxy(property)]
    fn set_brightness(&self, value: Brightness) -> zbus::Result<()>;

    /// BuiltinAnimations property
    #[dbus_proxy(property)]
    fn builtin_animations(&self) -> zbus::Result<Animations>;
    #[dbus_proxy(property)]
    fn set_builtin_animations(&self, value: Animations) -> zbus::Result<()>;

    /// BuiltinsEnabled property
    #[dbus_proxy(property)]
    fn builtins_enabled(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_builtins_enabled(&self, value: bool) -> zbus::Result<()>;

    /// EnableDisplay property
    #[dbus_proxy(property)]
    fn enable_display(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_enable_display(&self, value: bool) -> zbus::Result<()>;

    /// OffWhenLidClosed property
    #[dbus_proxy(property)]
    fn off_when_lid_closed(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_off_when_lid_closed(&self, value: bool) -> zbus::Result<()>;

    /// OffWhenSuspended property
    #[dbus_proxy(property)]
    fn off_when_suspended(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_off_when_suspended(&self, value: bool) -> zbus::Result<()>;

    /// OffWhenUnplugged property
    #[dbus_proxy(property)]
    fn off_when_unplugged(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_off_when_unplugged(&self, value: bool) -> zbus::Result<()>;
}

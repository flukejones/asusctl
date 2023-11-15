use rog_anime::usb::{AnimAwake, AnimBooting, AnimShutdown, AnimSleeping, Brightness};
use rog_anime::{AnimeDataBuffer, DeviceState as AnimeDeviceState};
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.asuslinux.Daemon",
    default_service = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux/Anime"
)]
trait Anime {
    /// Set the global base brightness
    fn set_brightness(&self, bright: Brightness) -> zbus::Result<()>;

    /// Set whether the AniMe will show boot, suspend, or off animations
    fn set_builtins_enabled(&self, enabled: bool) -> zbus::Result<()>;

    /// Set which builtin animation is used for each stage
    fn set_builtin_animations(
        &self,
        boot: AnimBooting,
        awake: AnimAwake,
        sleep: AnimSleeping,
        shutdown: AnimShutdown,
    ) -> zbus::Result<()>;

    /// Set whether the AniMe is displaying images/data
    fn set_enable_display(&self, status: bool) -> zbus::Result<()>;

    /// SetOffWhenLidClosed method
    fn set_off_when_lid_closed(&self, enabled: bool) -> zbus::Result<()>;

    /// SetOffWhenSuspended method
    fn set_off_when_suspended(&self, enabled: bool) -> zbus::Result<()>;

    /// SetOffWhenUnplugged method
    fn set_off_when_unplugged(&self, enabled: bool) -> zbus::Result<()>;

    /// Writes a data stream of length. Will force system thread to exit until
    /// it is restarted
    fn write(&self, input: AnimeDataBuffer) -> zbus::Result<()>;

    // #[dbus_proxy(property)]
    fn device_state(&self) -> zbus::Result<AnimeDeviceState>;

    /// NotifyDeviceState signal
    #[dbus_proxy(signal)]
    fn notify_device_state(&self, data: AnimeDeviceState) -> zbus::Result<()>;
}

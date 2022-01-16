use rog_anime::{AnimeDataBuffer, AnimePowerStates};
use zbus_macros::dbus_proxy;

#[dbus_proxy(
    interface = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux/Anime"
)]
trait Anime {
    /// Set whether the AniMe will show boot, suspend, or off animations
    fn set_boot_on_off(&self, status: bool) -> zbus::Result<()>;

    /// Set the global AniMe brightness
    fn set_brightness(&self, bright: f32) -> zbus::Result<()>;

    /// Set whether the AniMe is displaying images/data
    fn set_on_off(&self, status: bool) -> zbus::Result<()>;

    /// Writes a data stream of length. Will force system thread to exit until it is restarted
    fn write(&self, input: AnimeDataBuffer) -> zbus::Result<()>;

    /// Get status of if the AniMe LEDs are on
    #[dbus_proxy(property)]
    fn awake_enabled(&self) -> zbus::Result<bool>;

    /// Get the status of if factory system-status animations are enabled
    #[dbus_proxy(property)]
    fn boot_enabled(&self) -> zbus::Result<bool>;

    /// Notify listeners of the status of AniMe LED power and factory system-status animations
    #[dbus_proxy(signal)]
    fn power_states(&self, data: AnimePowerStates) -> zbus::Result<()>;
}

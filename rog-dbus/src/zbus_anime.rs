use rog_anime::{AnimeDataBuffer, AnimePowerStates};
use std::sync::mpsc::Sender;
use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy(
    interface = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux/Anime"
)]
trait Daemon {
    /// Set whether the AniMe will show boot, suspend, or off animations
    fn set_boot_on_off(&self, status: bool) -> zbus::Result<()>;

    /// Set the global AniMe brightness
    fn set_brightness(&self, bright: f32) -> zbus::Result<()>;

    /// Set whether the AniMe is displaying images/data
    fn set_on_off(&self, status: bool) -> zbus::Result<()>;

    /// Writes a data stream of length. Will force system thread to exit until it is restarted
    fn write(&self, input: &[u8]) -> zbus::Result<()>;

    /// Get status of if the AniMe LEDs are on
    #[dbus_proxy(property)]
    fn awake_enabled(&self) -> zbus::Result<bool>;

    /// Get the status of if factory system-status animations are enabled
    #[dbus_proxy(property)]
    fn boot_enabled(&self) -> zbus::Result<bool>;

    /// Notify listeners of the status of AniMe LED power and factory system-status animations
    #[dbus_proxy(signal)]
    fn notify_power_states(&self, data: AnimePowerStates) -> zbus::Result<()>;
}

pub struct AnimeProxy<'a>(DaemonProxy<'a>);

impl<'a> AnimeProxy<'a> {
    #[inline]
    pub fn new(conn: &Connection) -> Result<Self> {
        Ok(AnimeProxy(DaemonProxy::new(&conn)?))
    }

    #[inline]
    pub fn proxy(&self) -> &DaemonProxy<'a> {
        &self.0
    }

    /// Set whether the AniMe will show boot, suspend, or off animations
    #[inline]
    pub fn set_system_animations(&self, on: bool) -> Result<()> {
        self.0.set_boot_on_off(on)
    }

    /// Set whether the AniMe is displaying images/data
    #[inline]
    pub fn set_led_power(&self, on: bool) -> Result<()> {
        self.0.set_on_off(on)
    }

    /// Writes a data stream of length. Will force system thread to exit until it is restarted
    #[inline]
    pub fn write(&self, input: AnimeDataBuffer) -> Result<()> {
        self.0.write(input.get())
    }

    /// Get status of if the AniMe LEDs are on
    #[inline]
    pub fn awake_enabled(&self) -> Result<bool> {
        self.0.awake_enabled()
    }

    /// Get the status of if factory system-status animations are enabled
    #[inline]
    pub fn boot_enabled(&self) -> Result<bool> {
        self.0.boot_enabled()
    }

    #[inline]
    pub fn connect_notify_power_states(
        &self,
        send: Sender<AnimePowerStates>,
    ) -> zbus::fdo::Result<()> {
        self.0.connect_notify_power_states(move |data| {
            send.send(data)
                .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))?;
            Ok(())
        })
    }
}

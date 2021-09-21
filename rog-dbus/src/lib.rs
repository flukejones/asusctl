pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";

pub mod zbus_anime;
pub mod zbus_charge;
pub mod zbus_led;
pub mod zbus_profile;
pub mod zbus_rogbios;
pub mod zbus_supported;

use rog_anime::AnimePowerStates;
use rog_aura::{AuraEffect, LedPowerStates};
use rog_profiles::Profile;
use std::sync::mpsc::{channel, Receiver};
use zbus::{Connection, Result, SignalReceiver};

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct DbusProxies<'a> {
    anime: zbus_anime::AnimeProxy<'a>,
    charge: zbus_charge::ChargeProxy<'a>,
    led: zbus_led::LedProxy<'a>,
    profile: zbus_profile::ProfileProxy<'a>,
    rog_bios: zbus_rogbios::RogBiosProxy<'a>,
    supported: zbus_supported::SupportProxy<'a>,
}

impl<'a> DbusProxies<'a> {
    #[inline]
    pub fn new() -> Result<(Self, Connection)> {
        let conn = Connection::new_system()?;

        Ok((
            DbusProxies {
                anime: zbus_anime::AnimeProxy::new(&conn)?,
                led: zbus_led::LedProxy::new(&conn)?,
                charge: zbus_charge::ChargeProxy::new(&conn)?,
                profile: zbus_profile::ProfileProxy::new(&conn)?,
                rog_bios: zbus_rogbios::RogBiosProxy::new(&conn)?,
                supported: zbus_supported::SupportProxy::new(&conn)?,
            },
            conn,
        ))
    }

    pub fn setup_recv(&'a self, conn: Connection) -> SignalReceiver<'a, 'a> {
        let mut recv = SignalReceiver::new(conn);
        recv.receive_for(self.anime.proxy());
        recv.receive_for(self.led.proxy());
        recv.receive_for(self.charge.proxy());
        recv.receive_for(self.profile.proxy());
        recv.receive_for(self.rog_bios.proxy());
        recv.receive_for(self.supported.proxy());
        recv
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxy<'a> {
        &self.anime
    }

    pub fn charge(&self) -> &zbus_charge::ChargeProxy<'a> {
        &self.charge
    }

    pub fn led(&self) -> &zbus_led::LedProxy<'a> {
        &self.led
    }

    pub fn profile(&self) -> &zbus_profile::ProfileProxy<'a> {
        &self.profile
    }

    pub fn rog_bios(&self) -> &zbus_rogbios::RogBiosProxy<'a> {
        &self.rog_bios
    }

    pub fn supported(&self) -> &zbus_supported::SupportProxy<'a> {
        &self.supported
    }
}

// Signals separated out
pub struct Signals {
    pub profile: Receiver<Profile>,
    pub led_mode: Receiver<AuraEffect>,
    pub led_power_state: Receiver<LedPowerStates>,
    pub anime_power_state: Receiver<AnimePowerStates>,
    pub charge: Receiver<u8>,
    pub bios_gsync: Receiver<bool>,
    pub bios_sound: Receiver<bool>,
}

impl Signals {
    #[inline]
    pub fn new(proxies: &DbusProxies) -> Result<Self> {
        Ok(Signals {
            profile: {
                let (tx, rx) = channel();
                proxies.profile.connect_notify_profile(tx)?;
                rx
            },
            charge: {
                let (tx, rx) = channel();
                proxies.charge.connect_notify_charge(tx)?;
                rx
            },
            led_mode: {
                let (tx, rx) = channel();
                proxies.led.connect_notify_led(tx)?;
                rx
            },
            led_power_state: {
                let (tx, rx) = channel();
                proxies.led.connect_notify_power_states(tx)?;
                rx
            },
            anime_power_state: {
                let (tx, rx) = channel();
                proxies.anime.connect_notify_power_states(tx)?;
                rx
            },
            bios_gsync: {
                let (tx, rx) = channel();
                proxies.rog_bios.connect_notify_dedicated_graphic_mode(tx)?;
                rx
            },
            bios_sound: {
                let (tx, rx) = channel();
                proxies.rog_bios.connect_notify_post_boot_sound(tx)?;
                rx
            },
        })
    }
}

/// This is the main way to communicate with the DBUS interface
pub struct RogDbusClient<'a> {
    proxies: DbusProxies<'a>,
    signals: Signals,
}

impl<'a> RogDbusClient<'a> {
    #[inline]
    pub fn new() -> Result<(Self, Connection)> {
        let (proxies, conn) = DbusProxies::new()?;
        let signals = Signals::new(&proxies)?;

        Ok((RogDbusClient { proxies, signals }, conn))
    }

    pub fn proxies(&self) -> &DbusProxies {
        &self.proxies
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    pub fn setup_recv(&'a self, conn: Connection) -> SignalReceiver<'a, 'a> {
        let mut recv = SignalReceiver::new(conn);
        recv.receive_for(self.proxies.anime.proxy());
        recv.receive_for(self.proxies.led.proxy());
        recv.receive_for(self.proxies.charge.proxy());
        recv.receive_for(self.proxies.profile.proxy());
        recv.receive_for(self.proxies.rog_bios.proxy());
        recv.receive_for(self.proxies.supported.proxy());
        recv
    }
}

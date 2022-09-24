pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";

pub mod zbus_anime;
pub mod zbus_led;
pub mod zbus_platform;
pub mod zbus_power;
pub mod zbus_profile;
pub mod zbus_supported;

// use rog_anime::AnimePowerStates;
// use rog_aura::{AuraEffect, LedPowerStates};
// use rog_profiles::Profile;
// use std::sync::mpsc::{channel, Receiver};
use zbus::{blocking, Connection, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct DbusProxiesBlocking<'a> {
    anime: zbus_anime::AnimeProxyBlocking<'a>,
    charge: zbus_power::PowerProxyBlocking<'a>,
    led: zbus_led::LedProxyBlocking<'a>,
    profile: zbus_profile::ProfileProxyBlocking<'a>,
    rog_bios: zbus_platform::RogBiosProxyBlocking<'a>,
    supported: zbus_supported::SupportedProxyBlocking<'a>,
}

impl<'a> DbusProxiesBlocking<'a> {
    #[inline]
    pub fn new() -> Result<(Self, blocking::Connection)> {
        let conn = blocking::Connection::system()?;

        Ok((
            DbusProxiesBlocking {
                anime: zbus_anime::AnimeProxyBlocking::new(&conn)?,
                led: zbus_led::LedProxyBlocking::new(&conn)?,
                charge: zbus_power::PowerProxyBlocking::new(&conn)?,
                profile: zbus_profile::ProfileProxyBlocking::new(&conn)?,
                rog_bios: zbus_platform::RogBiosProxyBlocking::new(&conn)?,
                supported: zbus_supported::SupportedProxyBlocking::new(&conn)?,
            },
            conn,
        ))
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxyBlocking<'a> {
        &self.anime
    }

    pub fn charge(&self) -> &zbus_power::PowerProxyBlocking<'a> {
        &self.charge
    }

    pub fn led(&self) -> &zbus_led::LedProxyBlocking<'a> {
        &self.led
    }

    pub fn profile(&self) -> &zbus_profile::ProfileProxyBlocking<'a> {
        &self.profile
    }

    pub fn rog_bios(&self) -> &zbus_platform::RogBiosProxyBlocking<'a> {
        &self.rog_bios
    }

    pub fn supported(&self) -> &zbus_supported::SupportedProxyBlocking<'a> {
        &self.supported
    }
}

/// This is the main way to communicate with the DBUS interface
pub struct RogDbusClientBlocking<'a> {
    proxies: DbusProxiesBlocking<'a>,
}

impl<'a> RogDbusClientBlocking<'a> {
    #[inline]
    pub fn new() -> Result<(Self, blocking::Connection)> {
        let (proxies, conn) = DbusProxiesBlocking::new()?;
        Ok((RogDbusClientBlocking { proxies }, conn))
    }

    pub fn proxies(&self) -> &DbusProxiesBlocking {
        &self.proxies
    }
}

pub struct DbusProxies<'a> {
    anime: zbus_anime::AnimeProxy<'a>,
    charge: zbus_power::PowerProxy<'a>,
    led: zbus_led::LedProxy<'a>,
    profile: zbus_profile::ProfileProxy<'a>,
    rog_bios: zbus_platform::RogBiosProxy<'a>,
    supported: zbus_supported::SupportedProxy<'a>,
}

impl<'a> DbusProxies<'a> {
    #[inline]
    pub async fn new() -> Result<(DbusProxies<'a>, Connection)> {
        let conn = Connection::system().await?;

        Ok((
            DbusProxies {
                anime: zbus_anime::AnimeProxy::new(&conn).await?,
                led: zbus_led::LedProxy::new(&conn).await?,
                charge: zbus_power::PowerProxy::new(&conn).await?,
                profile: zbus_profile::ProfileProxy::new(&conn).await?,
                rog_bios: zbus_platform::RogBiosProxy::new(&conn).await?,
                supported: zbus_supported::SupportedProxy::new(&conn).await?,
            },
            conn,
        ))
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxy<'a> {
        &self.anime
    }

    pub fn charge(&self) -> &zbus_power::PowerProxy<'a> {
        &self.charge
    }

    pub fn led(&self) -> &zbus_led::LedProxy<'a> {
        &self.led
    }

    pub fn profile(&self) -> &zbus_profile::ProfileProxy<'a> {
        &self.profile
    }

    pub fn rog_bios(&self) -> &zbus_platform::RogBiosProxy<'a> {
        &self.rog_bios
    }

    pub fn supported(&self) -> &zbus_supported::SupportedProxy<'a> {
        &self.supported
    }
}

/// This is the main way to communicate with the DBUS interface
pub struct RogDbusClient<'a> {
    proxies: DbusProxies<'a>,
}

impl<'a> RogDbusClient<'a> {
    #[inline]
    pub async fn new() -> Result<(RogDbusClient<'a>, Connection)> {
        let (proxies, conn) = DbusProxies::new().await?;
        Ok((RogDbusClient { proxies }, conn))
    }

    pub fn proxies(&self) -> &DbusProxies {
        &self.proxies
    }
}

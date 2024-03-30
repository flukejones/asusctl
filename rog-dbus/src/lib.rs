pub use asusd::{DBUS_IFACE, DBUS_NAME, DBUS_PATH};

pub mod zbus_anime;
pub mod zbus_aura;
pub mod zbus_fan_curves;
pub mod zbus_platform;
pub mod zbus_slash;

// use rog_anime::AnimePowerStates;
// use rog_aura::{AuraEffect, LedPowerStates};
// use rog_profiles::Profile;
// use std::sync::mpsc::{channel, Receiver};
use zbus::{blocking, Connection, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct DbusProxiesBlocking<'a> {
    anime: zbus_anime::AnimeProxyBlocking<'a>,
    slash: zbus_slash::SlashProxyBlocking<'a>,
    led: zbus_aura::AuraProxyBlocking<'a>,
    profile: zbus_fan_curves::FanCurvesProxyBlocking<'a>,
    rog_bios: zbus_platform::PlatformProxyBlocking<'a>,
}

impl<'a> DbusProxiesBlocking<'a> {
    #[inline]
    pub fn new() -> Result<(Self, blocking::Connection)> {
        let conn = blocking::Connection::system()?;

        Ok((
            DbusProxiesBlocking {
                anime: zbus_anime::AnimeProxyBlocking::new(&conn)?,
                slash: zbus_slash::SlashProxyBlocking::new(&conn)?,
                led: zbus_aura::AuraProxyBlocking::new(&conn)?,
                profile: zbus_fan_curves::FanCurvesProxyBlocking::new(&conn)?,
                rog_bios: zbus_platform::PlatformProxyBlocking::new(&conn)?,
            },
            conn,
        ))
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxyBlocking<'a> {
        &self.anime
    }
    pub fn slash(&self) -> &zbus_slash::SlashProxyBlocking<'a> { &self.slash }
    pub fn aura(&self) -> &zbus_aura::AuraProxyBlocking<'a> {
        &self.led
    }
    pub fn fan_curves(&self) -> &zbus_fan_curves::FanCurvesProxyBlocking<'a> {
        &self.profile
    }
    pub fn platform(&self) -> &zbus_platform::PlatformProxyBlocking<'a> {
        &self.rog_bios
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

    pub fn proxies(&self) -> &DbusProxiesBlocking<'_> {
        &self.proxies
    }
}

pub struct DbusProxies<'a> {
    anime: zbus_anime::AnimeProxy<'a>,
    slash: zbus_slash::SlashProxy<'a>,
    led: zbus_aura::AuraProxy<'a>,
    profile: zbus_fan_curves::FanCurvesProxy<'a>,
    rog_bios: zbus_platform::PlatformProxy<'a>,
}

impl<'a> DbusProxies<'a> {
    #[inline]
    pub async fn new() -> Result<(DbusProxies<'a>, Connection)> {
        let conn = Connection::system().await?;

        Ok((
            DbusProxies {
                anime: zbus_anime::AnimeProxy::new(&conn).await?,
                slash: zbus_slash::SlashProxy::new(&conn).await?,
                led: zbus_aura::AuraProxy::new(&conn).await?,
                profile: zbus_fan_curves::FanCurvesProxy::new(&conn).await?,
                rog_bios: zbus_platform::PlatformProxy::new(&conn).await?,
            },
            conn,
        ))
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxy<'a> { &self.anime }
    pub fn slash(&self) -> &zbus_slash::SlashProxy<'a> { &self.slash }

    pub fn led(&self) -> &zbus_aura::AuraProxy<'a> {
        &self.led
    }

    pub fn profile(&self) -> &zbus_fan_curves::FanCurvesProxy<'a> {
        &self.profile
    }

    pub fn rog_bios(&self) -> &zbus_platform::PlatformProxy<'a> {
        &self.rog_bios
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

    pub fn proxies(&self) -> &DbusProxies<'_> {
        &self.proxies
    }
}

pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";

pub mod zbus_anime;
pub mod zbus_charge;
pub mod zbus_gfx;
pub mod zbus_led;
pub mod zbus_profile;
pub mod zbus_rogbios;
pub mod zbus_supported;

use rog_anime::AnimePowerStates;
use rog_aura::{AuraEffect, LedPowerStates};
use rog_profiles::profiles::Profile;
use rog_types::gfx_vendors::{GfxRequiredUserAction, GfxVendors};
use std::sync::mpsc::{channel, Receiver};
use zbus::{Connection, Result, SignalReceiver};

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct DbusProxies<'a> {
    anime: zbus_anime::AnimeProxy<'a>,
    charge: zbus_charge::ChargeProxy<'a>,
    gfx: zbus_gfx::GfxProxy<'a>,
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
                gfx: zbus_gfx::GfxProxy::new(&conn)?,
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
        recv.receive_for(self.gfx.proxy());
        recv.receive_for(self.profile.proxy());
        recv
    }

    pub fn anime(&self) -> &zbus_anime::AnimeProxy<'a> {
        &self.anime
    }

    pub fn charge(&self) -> &zbus_charge::ChargeProxy<'a> {
        &self.charge
    }

    pub fn gfx(&self) -> &zbus_gfx::GfxProxy<'a> {
        &self.gfx
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
    pub gfx_vendor: Receiver<GfxVendors>,
    pub gfx_action: Receiver<GfxRequiredUserAction>,
    pub profile: Receiver<Profile>,
    pub led_mode: Receiver<AuraEffect>,
    pub led_power_state: Receiver<LedPowerStates>,
    pub anime_power_state: Receiver<AnimePowerStates>,
    pub charge: Receiver<u8>,
}

impl Signals {
    #[inline]
    pub fn new(proxies: &DbusProxies) -> Result<Self> {
        Ok(Signals {
            gfx_vendor: {
                let (tx, rx) = channel();
                proxies.gfx.connect_notify_gfx(tx)?;
                rx
            },
            gfx_action: {
                let (tx, rx) = channel();
                proxies.gfx.connect_notify_action(tx)?;
                rx
            },
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
        recv.receive_for(self.proxies.gfx.proxy());
        recv.receive_for(self.proxies.profile.proxy());
        recv
    }

    /*
     * GFX
     */
    pub fn gfx_wait_changed(&self) -> Result<GfxRequiredUserAction> {
        loop {
            if let Ok(res) = self.proxies.gfx.proxy().next_signal() {
                if res.is_none() {
                    if let Ok(stuff) = self.signals.gfx_action.try_recv() {
                        return Ok(stuff);
                    }
                    // return Ok("Failed for unknown reason".to_owned());
                }
            }
        }
    }
}

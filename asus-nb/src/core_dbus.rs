use super::*;
use std::sync::{Arc, Mutex};
use zbus::{Connection, Result, SignalReceiver};

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

    pub fn setup_recv(&'a self, conn: Connection) -> SignalReceiver {
        let mut recv = SignalReceiver::new(conn);
        //recv.receive_for(&self.proxy_anime);
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
    pub gfx_vendor: Arc<Mutex<Option<String>>>,
    pub gfx_action: Arc<Mutex<Option<String>>>,
    pub profile: Arc<Mutex<Option<String>>>,
    pub led_mode: Arc<Mutex<Option<AuraModes>>>,
    pub charge: Arc<Mutex<Option<u8>>>,
}

impl Signals {
    #[inline]
    pub fn new(proxies: &DbusProxies) -> Result<Self> {
        //
        let charge_signal = Arc::new(Mutex::new(None));
        proxies
            .charge
            .connect_notify_charge(charge_signal.clone())?;

        //
        let ledmode_signal = Arc::new(Mutex::new(None));
        proxies.led.connect_notify_led(ledmode_signal.clone())?;

        let gfx_action_signal = Arc::new(Mutex::new(None));
        proxies
            .gfx
            .connect_notify_action(gfx_action_signal.clone())?;

        let gfx_vendor_signal = Arc::new(Mutex::new(None));
        proxies.gfx.connect_notify_gfx(gfx_vendor_signal.clone())?;

        let profile_signal = Arc::new(Mutex::new(None));
        proxies
            .profile
            .connect_notify_profile(profile_signal.clone())?;

        Ok(Signals {
            gfx_vendor: gfx_vendor_signal,
            gfx_action: gfx_action_signal,
            profile: profile_signal,
            led_mode: ledmode_signal,
            charge: charge_signal,
        })
    }
}

/// This is the main way to communicate with the DBUS interface
pub struct AuraDbusClient<'a> {
    proxies: DbusProxies<'a>,
    signals: Signals,
}

impl<'a> AuraDbusClient<'a> {
    #[inline]
    pub fn new() -> Result<(Self, Connection)> {
        let (proxies, conn) = DbusProxies::new()?;
        let signals = Signals::new(&proxies)?;

        Ok((AuraDbusClient { proxies, signals }, conn))
    }

    pub fn proxies(&self) -> &DbusProxies {
        &self.proxies
    }

    /*
     * GFX
     */
    pub fn gfx_wait_changed(&self) -> Result<String> {
        loop {
            if let Ok(res) = self.proxies.gfx.proxy().next_signal() {
                if res.is_none() {
                    if let Ok(lock) = self.signals.gfx_action.lock() {
                        if let Some(stuff) = lock.as_ref() {
                            return Ok(stuff.to_string());
                        }
                    }
                    // return Ok("Failed for unknown reason".to_owned());
                }
            }
        }
    }
}

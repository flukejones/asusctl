use super::*;
use crate::fancy::KeyColourArray;
use crate::profile::ProfileEvent;
use ctrl_gfx::vendors::GfxVendors;
use dbus::channel::Sender;
use dbus::{blocking::Connection, Message};
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{thread, time::Duration};

use crate::dbus_gfx::OrgAsuslinuxDaemonNotifyGfx;
use crate::dbus_ledmode::OrgAsuslinuxDaemonNotifyLed;
use crate::dbus_profile::OrgAsuslinuxDaemonNotifyProfile;
use crate::dbus_charge::OrgAsuslinuxDaemonNotifyCharge;

// Signals separated out
pub struct CtrlSignals {
    pub gfx_signal: Arc<Mutex<Option<String>>>,
    pub profile_signal: Arc<Mutex<Option<String>>>,
    pub ledmode_signal: Arc<Mutex<Option<AuraModes>>>,
    pub charge_signal: Arc<Mutex<Option<u8>>>,
}

impl CtrlSignals {
    #[inline]
    pub fn new(connection: &Connection) -> Result<Self, Box<dyn Error>> {
        let proxy = connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Gfx",
            Duration::from_millis(5000),
        );

        let gfx_signal = Arc::new(Mutex::new(None));
        let gfx_res1 = gfx_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyGfx, _: &Connection, _: &Message| {
                if let Ok(mut lock) = gfx_res1.lock() {
                    *lock = Some(sig.vendor);
                }
                true
            },
        )?;

        //
        let proxy = connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Profile",
            Duration::from_millis(5000),
        );

        let profile_signal = Arc::new(Mutex::new(None));
        let prof_res1 = profile_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyProfile, _: &Connection, _: &Message| {
                if let Ok(mut lock) = prof_res1.lock() {
                    *lock = Some(sig.profile);
                }
                true
            },
        )?;

        //
        let proxy = connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_millis(5000),
        );

        let ledmode_signal = Arc::new(Mutex::new(None));
        let led_res1 = ledmode_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyLed, _: &Connection, _: &Message| {
                if let Ok(mut lock) = led_res1.lock() {
                    if let Ok(dat) = serde_json::from_str(&sig.data) {
                        *lock = Some(dat);
                    }
                }
                true
            },
        )?;

        //
        let proxy = connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Charge",
            Duration::from_millis(5000),
        );

        let charge_signal = Arc::new(Mutex::new(None));
        let charge_res1 = charge_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyCharge, _: &Connection, _: &Message| {
                if let Ok(mut lock) = charge_res1.lock() {
                        *lock = Some(sig.limit);
                }
                true
            },
        )?;

        Ok(CtrlSignals {
            gfx_signal,
            profile_signal,
            ledmode_signal,
            charge_signal,
        })
    }
}

/// Simplified way to write a effect block
pub struct AuraDbusClient {
    connection: Box<Connection>,
    block_time: u64,
    stop: Arc<AtomicBool>,
    signals: CtrlSignals,
}

impl AuraDbusClient {
    #[inline]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = Connection::new_system()?;

        let stop = Arc::new(AtomicBool::new(false));
        let match_rule = dbus::message::MatchRule::new_signal(DBUS_IFACE, "NotifyLed");
        let stop1 = stop.clone();
        connection.add_match(match_rule, move |_: (), _, msg| {
            if msg.read1::<&str>().is_ok() {
                stop1.clone().store(true, Ordering::Relaxed);
            }
            true
        })?;

        let signals = CtrlSignals::new(&connection)?;

        Ok(AuraDbusClient {
            connection: Box::new(connection),
            block_time: 33333,
            stop,
            signals,
        })
    }

    pub fn wait_gfx_changed(&self) -> Result<String, Box<dyn Error>> {
        loop {
            self.connection.process(Duration::from_micros(500))?;
            if let Ok(lock) = self.signals.gfx_signal.lock() {
                if let Some(stuff) = lock.as_ref() {
                    return Ok(stuff.to_string());
                }
            }
        }
    }

    /// This method must always be called before the very first write to initialise
    /// the keyboard LED EC in the correct mode
    #[inline]
    pub fn init_effect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mode = AuraModes::PerKey(vec![vec![]]);
        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Led", DBUS_IFACE, "SetLedMode")?
                .append1(serde_json::to_string(&mode)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    /// Write a single colour block.
    ///
    /// Intentionally blocks for 10ms after sending to allow the block to
    /// be written to the keyboard EC. This should not be async.
    #[inline]
    pub fn write_colour_block(
        &mut self,
        key_colour_array: &KeyColourArray,
    ) -> Result<(), Box<dyn Error>> {
        let group = key_colour_array.get();
        let mut vecs = Vec::with_capacity(group.len());
        for v in group {
            vecs.push(v.to_vec());
        }
        let mode = AuraModes::PerKey(vecs);
        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Led", DBUS_IFACE, "SetLedMode")?
                .append1(serde_json::to_string(&mode)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        thread::sleep(Duration::from_micros(self.block_time));
        self.connection.process(Duration::from_micros(500))?;
        if self.stop.load(Ordering::Relaxed) {
            println!("Keyboard backlight was changed, exiting");
            std::process::exit(1)
        }
        Ok(())
    }

    #[inline]
    pub fn write_keyboard_leds(&self, mode: &AuraModes) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Led", DBUS_IFACE, "SetLedMode")?
                .append1(serde_json::to_string(mode)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_fan_mode(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg = Message::new_method_call(
            DBUS_NAME,
            "/org/asuslinux/Profile",
            DBUS_IFACE,
            "SetProfile",
        )?
        .append1(serde_json::to_string(&ProfileEvent::ChangeMode(level))?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_gfx_mode(&self, vendor: GfxVendors) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Gfx", DBUS_IFACE, "SetVendor")?
                .append1(<&str>::from(&vendor));
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_profile_command(
        &self,
        cmd: &ProfileEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg = Message::new_method_call(
            DBUS_NAME,
            "/org/asuslinux/Profile",
            DBUS_IFACE,
            "SetProfile",
        )?
        .append1(serde_json::to_string(cmd)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_charge_limit(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Charge", DBUS_IFACE, "SetLimit")?
                .append1(level);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_builtin_mode(&self, mode: &AuraModes) -> Result<(), Box<dyn std::error::Error>> {
        self.write_keyboard_leds(mode)
    }

    #[inline]
    pub fn write_brightness(&self, level: u8) -> Result<String, Box<dyn std::error::Error>> {
        self.write_keyboard_leds(&AuraModes::LedBrightness(level))?;
        Ok(String::new())
    }
}

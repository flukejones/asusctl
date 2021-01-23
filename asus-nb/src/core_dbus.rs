use super::*;
use crate::cli_options::LedBrightness;
use crate::fancy::KeyColourArray;
use crate::profile::ProfileEvent;
use dbus::{blocking::Connection, Message};
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{thread, time::Duration};

use crate::dbus_charge::{
    OrgAsuslinuxDaemon as OrgAsuslinuxDaemonCharge, OrgAsuslinuxDaemonNotifyCharge,
};
use crate::dbus_gfx::{
    OrgAsuslinuxDaemon as OrgAsuslinuxDaemonGfx, OrgAsuslinuxDaemonNotifyAction,
    OrgAsuslinuxDaemonNotifyGfx,
};
use crate::dbus_ledmode::{
    OrgAsuslinuxDaemon as OrgAsuslinuxDaemonLed, OrgAsuslinuxDaemonNotifyLed,
};
use crate::dbus_profile::{
    OrgAsuslinuxDaemon as OrgAsuslinuxDaemonProfile, OrgAsuslinuxDaemonNotifyProfile,
};
use crate::dbus_rogbios::OrgAsuslinuxDaemon as OrgAsuslinuxDaemonRogBios;
use crate::dbus_supported::OrgAsuslinuxDaemon as OrgAsuslinuxDaemonSupported;

// Signals separated out
pub struct CtrlSignals {
    pub gfx_vendor_signal: Arc<Mutex<Option<String>>>,
    pub gfx_action_signal: Arc<Mutex<Option<String>>>,
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
            Duration::from_secs(2),
        );

        let gfx_vendor_signal = Arc::new(Mutex::new(None));
        let gfx_res1 = gfx_vendor_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyGfx, _: &Connection, _: &Message| {
                if let Ok(mut lock) = gfx_res1.lock() {
                    *lock = Some(sig.vendor);
                }
                true
            },
        )?;

        let gfx_action_signal = Arc::new(Mutex::new(None));
        let gfx_res1 = gfx_action_signal.clone();

        let _x = proxy.match_signal(
            move |sig: OrgAsuslinuxDaemonNotifyAction, _: &Connection, _: &Message| {
                if let Ok(mut lock) = gfx_res1.lock() {
                    *lock = Some(sig.action);
                }
                true
            },
        )?;

        //
        let proxy = connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Profile",
            Duration::from_secs(2),
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
            Duration::from_secs(2),
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
            Duration::from_secs(2),
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
            gfx_vendor_signal,
            gfx_action_signal,
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
            self.connection.process(Duration::from_millis(1))?;
            if let Ok(lock) = self.signals.gfx_action_signal.lock() {
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
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_secs(2),
        );
        proxy.set_led_mode(&serde_json::to_string(&mode)?)?;
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

        self.write_keyboard_leds(&mode)?;

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
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_secs(2),
        );
        proxy.set_led_mode(&serde_json::to_string(mode)?)?;
        Ok(())
    }

    #[inline]
    pub fn next_keyboard_led_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_secs(2),
        );
        proxy.next_led_mode()?;
        Ok(())
    }

    #[inline]
    pub fn prev_keyboard_led_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_secs(2),
        );
        proxy.prev_led_mode()?;
        Ok(())
    }

    #[inline]
    pub fn get_gfx_pwr(&self) -> Result<String, Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Gfx",
            Duration::from_secs(2),
        );
        let x = proxy.power()?;
        Ok(x)
    }

    #[inline]
    pub fn get_gfx_mode(&self) -> Result<String, Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Gfx",
            Duration::from_secs(2),
        );
        let x = proxy.vendor()?;
        Ok(x)
    }

    #[inline]
    pub fn write_gfx_mode(&self, vendor: String) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Gfx",
            Duration::from_secs(30),
        );
        proxy.set_vendor(&vendor)?;
        Ok(())
    }

    #[inline]
    pub fn next_fan_profile(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Profile",
            Duration::from_secs(2),
        );
        proxy.next_profile()?;
        Ok(())
    }

    #[inline]
    pub fn write_fan_mode(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Profile",
            Duration::from_secs(2),
        );
        proxy.set_profile(&serde_json::to_string(&ProfileEvent::ChangeMode(level))?)?;
        Ok(())
    }

    #[inline]
    pub fn write_profile_command(
        &self,
        cmd: &ProfileEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Profile",
            Duration::from_secs(2),
        );
        proxy.set_profile(&serde_json::to_string(cmd)?)?;
        Ok(())
    }

    #[inline]
    pub fn write_charge_limit(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Charge",
            Duration::from_secs(2),
        );
        proxy.set_limit(level)?;
        Ok(())
    }

    #[inline]
    pub fn write_builtin_mode(&self, mode: &AuraModes) -> Result<(), Box<dyn std::error::Error>> {
        self.write_keyboard_leds(mode)
    }

    #[inline]
    pub fn get_led_brightness(&self) -> Result<LedBrightness, Box<dyn Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Led",
            Duration::from_secs(2),
        );
        match proxy.led_brightness()? {
            -1 => Ok(LedBrightness::new(None)),
            level => Ok(LedBrightness::new(Some(level as u8))),
        }
    }

    #[inline]
    pub fn write_brightness(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.write_keyboard_leds(&AuraModes::LedBrightness(level))?;
        Ok(())
    }

    //
    #[inline]
    pub fn get_bios_dedicated_gfx(&self) -> Result<i16, Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/RogBios",
            Duration::from_secs(2),
        );
        let x = proxy.dedicated_graphic_mode()?;
        Ok(x)
    }

    #[inline]
    pub fn set_bios_dedicated_gfx(&self, on: bool) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/RogBios",
            Duration::from_secs(2),
        );
        proxy.set_dedicated_graphic_mode(<bool>::from(on))?;
        Ok(())
    }

    #[inline]
    pub fn get_bios_post_sound(&self) -> Result<i16, Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/RogBios",
            Duration::from_secs(2),
        );
        let x = proxy.post_boot_sound()?;
        Ok(x)
    }

    #[inline]
    pub fn set_bios_post_sound(&self, on: bool) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/RogBios",
            Duration::from_secs(2),
        );
        proxy.set_post_boot_sound(<bool>::from(on))?;
        Ok(())
    }

    #[inline]
    pub fn get_supported_functions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let proxy = self.connection.with_proxy(
            "org.asuslinux.Daemon",
            "/org/asuslinux/Supported",
            Duration::from_secs(2),
        );
        let x = proxy.supported_functions()?;
        Ok(x)
    }
}

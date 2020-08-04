use super::*;
use crate::fancy::KeyColourArray;
use dbus::channel::Sender;
use dbus::{blocking::Connection, channel::Token, Message};
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{thread, time::Duration};

/// Simplified way to write a effect block
pub struct AuraDbusClient {
    connection: Box<Connection>,
    block_time: u64,
    stop: Arc<AtomicBool>,
    stop_token: Token,
}

impl AuraDbusClient {
    #[inline]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = Connection::new_system()?;
        let stop = Arc::new(AtomicBool::new(false));

        let stopper2 = stop.clone();
        let match_rule = dbus::message::MatchRule::new_signal(DBUS_IFACE, "KeyBacklightChanged");
        let stop_token = connection.add_match(match_rule, move |_: (), _, msg| {
            if msg.read1::<&str>().is_ok() {
                stopper2.store(true, Ordering::Relaxed);
            }
            true
        })?;

        Ok(AuraDbusClient {
            connection: Box::new(connection),
            block_time: 33333,
            stop,
            stop_token,
        })
    }

    /// This method must always be called before the very first write to initialise
    /// the keyboard LED EC in the correct mode
    #[inline]
    pub fn init_effect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mode = AuraModes::RGB(vec![vec![]]);
        let mut msg =
            Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "SetKeyBacklight")?
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
        let mode = AuraModes::RGB(vecs);
        let mut msg =
            Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "SetKeyBacklight")?
                .append1(serde_json::to_string(&mode)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        thread::sleep(Duration::from_micros(self.block_time));
        self.connection.process(Duration::from_micros(500))?;
        if self.stop.load(Ordering::Relaxed) {
            self.connection.remove_match(self.stop_token)?;
            println!("Keyboard backlight was changed, exiting");
            std::process::exit(1)
        }
        Ok(())
    }

    #[inline]
    pub fn write_keyboard_leds(&self, mode: &AuraModes) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg =
            Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "SetKeyBacklight")?
                .append1(serde_json::to_string(mode)?);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_fan_mode(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg = Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "SetFanMode")?
            .append1(level);
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        Ok(())
    }

    #[inline]
    pub fn write_charge_limit(&self, level: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg = Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "SetChargeLimit")?
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

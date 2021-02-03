use rog_types::{
    fancy::{Key, KeyColourArray},
};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (dbus, _) = AuraDbusClient::new()?;

    let mut key_colours = KeyColourArray::new();

    dbus.proxies().led().init_effect()?;
    loop {
        let count = 49;
        for _ in 0..count {
            *key_colours.key(Key::ROG).unwrap().0 += 5;
            *key_colours.key(Key::L).unwrap().0 += 5;
            *key_colours.key(Key::I).unwrap().0 += 5;
            *key_colours.key(Key::N).unwrap().0 += 5;
            *key_colours.key(Key::U).unwrap().0 += 5;
            *key_colours.key(Key::X).unwrap().0 += 5;
            dbus.proxies().led().set_per_key(&key_colours)?;
        }
        for _ in 0..count {
            *key_colours.key(Key::ROG).unwrap().0 -= 5;
            *key_colours.key(Key::L).unwrap().0 -= 5;
            *key_colours.key(Key::I).unwrap().0 -= 5;
            *key_colours.key(Key::N).unwrap().0 -= 5;
            *key_colours.key(Key::U).unwrap().0 -= 5;
            *key_colours.key(Key::X).unwrap().0 -= 5;
            dbus.proxies().led().set_per_key(&key_colours)?;
        }
    }
}

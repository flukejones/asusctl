//! Using a combination of key-colour array plus a key layout to generate outputs.

use rog_aura::{keys::Key, layouts::KeyLayout, ActionData, Colour, LedType, Sequences, Speed};
use rog_dbus::RogDbusClientBlocking;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layout = KeyLayout::gx502_layout();

    let (client, _) = RogDbusClientBlocking::new().unwrap();

    let mut seq = Sequences::new();
    let mut key = ActionData::new_breathe(
        LedType::Key(Key::W),
        Colour(255, 127, 0),
        Colour(127, 0, 255),
        Speed::Med,
    );

    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::A));
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::S));
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::D));
    seq.push(key.clone());

    let mut key = ActionData::new_breathe(
        LedType::Key(Key::Q),
        Colour(127, 127, 127),
        Colour(127, 255, 255),
        Speed::Low,
    );
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::E));
    seq.push(key.clone());

    let mut key = ActionData::new_breathe(
        LedType::Key(Key::N1),
        Colour(166, 127, 166),
        Colour(127, 155, 20),
        Speed::High,
    );
    key.set_led_type(LedType::Key(Key::Tilde));
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::N2));
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::N3));
    seq.push(key.clone());
    key.set_led_type(LedType::Key(Key::N4));
    seq.push(key.clone());

    loop {
        seq.next_state(&layout);
        let packets = seq.create_packets();

        client.proxies().led().per_key_raw(packets)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
}

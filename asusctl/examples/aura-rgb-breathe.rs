//! Using a combination of key-colour array plus a key layout to generate outputs.

use rog_aura::{keys::Key, layouts::KeyLayout, ActionData, Colour, Sequences, Speed};
use rog_dbus::RogDbusClientBlocking;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layout = KeyLayout::gx502_layout();

    let (client, _) = RogDbusClientBlocking::new().unwrap();

    let mut seq = Sequences::new();
    let mut key =
        ActionData::new_breathe(Key::W, Colour(255, 127, 0), Colour(127, 0, 255), Speed::Med);

    seq.push(key.clone());
    key.set_key(Key::A);
    seq.push(key.clone());
    key.set_key(Key::S);
    seq.push(key.clone());
    key.set_key(Key::D);
    seq.push(key.clone());

    let mut key = ActionData::new_breathe(
        Key::Q,
        Colour(127, 127, 127),
        Colour(127, 255, 255),
        Speed::Low,
    );
    seq.push(key.clone());
    key.set_key(Key::E);
    seq.push(key.clone());

    let mut key = ActionData::new_breathe(
        Key::N1,
        Colour(166, 127, 166),
        Colour(127, 155, 20),
        Speed::High,
    );
    key.set_key(Key::Tilde);
    seq.push(key.clone());
    key.set_key(Key::N2);
    seq.push(key.clone());
    key.set_key(Key::N3);
    seq.push(key.clone());
    key.set_key(Key::N4);
    seq.push(key.clone());

    loop {
        seq.next_state(&layout);
        let packets = seq.create_packets();
        println!("{:#0x?}", &packets[0]);

        client.proxies().led().per_key_raw(packets)?;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}

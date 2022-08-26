//! Using a combination of key-colour array plus a key layout to generate outputs.

use rog_aura::{layouts::KeyLayout, ActionData, Colour, LedType, PerZone, Sequences, Speed};
use rog_dbus::RogDbusClientBlocking;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layout = KeyLayout::gx502_layout();

    let (client, _) = RogDbusClientBlocking::new().unwrap();

    let mut seq = Sequences::new();

    let zone = ActionData::new_breathe(
        LedType::Zone(PerZone::KeyboardLeft),
        Colour(166, 127, 166),
        Colour(127, 155, 20),
        Speed::High,
    );
    seq.push(zone);

    let zone = ActionData::new_breathe(
        LedType::Zone(PerZone::KeyboardCenterLeft),
        Colour(16, 127, 255),
        Colour(127, 15, 20),
        Speed::Low,
    );
    seq.push(zone);

    let zone = ActionData::new_breathe(
        LedType::Zone(PerZone::LightbarRightCorner),
        Colour(0, 255, 255),
        Colour(255, 0, 255),
        Speed::Med,
    );
    seq.push(zone);

    loop {
        seq.next_state(&layout);
        let packets = seq.create_packets();

        client.proxies().led().per_key_raw(packets)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
}

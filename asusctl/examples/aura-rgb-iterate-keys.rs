//! Using a combination of key-colour array plus a key layout to generate outputs.

use rog_aura::{layouts::KeyLayout, KeyColourArray};
use rog_dbus::RogDbusClientBlocking;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, _) = RogDbusClientBlocking::new().unwrap();
    let layout = KeyLayout::gx502_layout();
    loop {
        let mut key_colours = KeyColourArray::new();
        for row in layout.rows() {
            for (k, key) in row.row().enumerate() {
                if k != 0 {
                    if let Some(prev) = row.row().nth(k - 1) {
                        if let Some(c) = key_colours.rgb_for_key(*prev) {
                            c[0] = 0;
                        };
                    }
                }

                if key.is_placeholder() {
                    continue;
                }

                if let Some(c) = key_colours.rgb_for_key(*key) {
                    c[0] = 255;
                };

                client.proxies().led().per_key_raw(key_colours.get())?;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

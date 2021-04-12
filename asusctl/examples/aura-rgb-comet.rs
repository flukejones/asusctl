use rog_aura::{GX502Layout, KeyColourArray, KeyLayout};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (dbus, _) = AuraDbusClient::new()?;

    let layout = GX502Layout::default();

    dbus.proxies().led().init_effect()?;
    let rows = layout.get_rows();

    let mut column = 0;
    loop {
        let mut key_colours = KeyColourArray::new();
        for row in rows {
            if let Some(c) = key_colours.key(row[column as usize]) {
                *c.0 = 255;
            };
        }
        if column == rows[0].len() - 1 {
            column = 0
        } else {
            column += 1;
        }

        dbus.proxies().led().set_per_key(&key_colours)?;
    }
}

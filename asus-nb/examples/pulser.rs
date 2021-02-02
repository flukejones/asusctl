use asus_nb::{
    core_dbus::AuraDbusClient,
    fancy::{GX502Layout, KeyColourArray, KeyLayout},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (dbus, _) = AuraDbusClient::new()?;

    let mut key_colours = KeyColourArray::new();
    let layout = GX502Layout::default();

    dbus.proxies().led().init_effect()?;
    let rows = layout.get_rows();

    let mut fade = 50;
    let mut flip = false;
    loop {
        for row in rows {
            for (k, key) in row.iter().enumerate() {
                if let Some(c) = key_colours.key(*key) {
                    *c.0 = 255 / fade / (k + 1) as u8;
                };
            }
        }

        dbus.proxies().led().set_per_key(&key_colours)?;

        if flip {
            if fade > 1 {
                fade -= 1;
            } else {
                flip = !flip;
            }
        } else if fade < 17 {
            fade += 1;
        } else {
            flip = !flip;
        }
    }
}

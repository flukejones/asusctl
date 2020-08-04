use asus_nb::{
    core_dbus::AuraDbusClient,
    fancy::{GX502Layout, KeyColourArray, KeyLayout},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = AuraDbusClient::new()?;

    let layout = GX502Layout::default();

    writer.init_effect()?;
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

        writer.write_colour_block(&key_colours)?;
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

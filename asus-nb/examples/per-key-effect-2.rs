use asus_nb::{
    core_dbus::AuraDbusWriter,
    fancy::{Key, KeyColourArray},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = AuraDbusWriter::new()?;

    let mut key_colours = KeyColourArray::new();

    writer.init_effect()?;
    loop {
        let count = 49;
        for _ in 0..count {
            *key_colours.key(Key::ROG).unwrap().0 += 5;
            *key_colours.key(Key::L).unwrap().0 += 5;
            *key_colours.key(Key::I).unwrap().0 += 5;
            *key_colours.key(Key::N).unwrap().0 += 5;
            *key_colours.key(Key::U).unwrap().0 += 5;
            *key_colours.key(Key::X).unwrap().0 += 5;
            writer.write_colour_block(&key_colours)?;
        }
        for _ in 0..count {
            *key_colours.key(Key::ROG).unwrap().0 -= 5;
            *key_colours.key(Key::L).unwrap().0 -= 5;
            *key_colours.key(Key::I).unwrap().0 -= 5;
            *key_colours.key(Key::N).unwrap().0 -= 5;
            *key_colours.key(Key::U).unwrap().0 -= 5;
            *key_colours.key(Key::X).unwrap().0 -= 5;
            writer.write_colour_block(&key_colours)?;
        }
    }
}

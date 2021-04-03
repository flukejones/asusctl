# rog-anime

## Features

`zbus` is enabled by default.

## Example

```rust
use std::{env, error::Error, path::Path, process::exit};

use rog_dbus::AuraDbusClient;
use rog_anime::{
    anime_data::AniMeDataBuffer,
    anime_image::{AnimeImage, Vec2},
};

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 8 {
        println!(
            "Usage: <filepath> <x scale> <y scale> <angle> <x pos> <y pos> <fineness> <brightness>"
        );
        println!("e.g, asusctl/examples/doom_large.png 0.9 0.9 0.4 0.0 0.0, 0.8");
        println!("All args except path and fineness are floats");
        exit(-1);
    }

    let image = AnimeImage::from_png(
        Path::new(&args[1]),
        Vec2::new(
            args[2].parse::<f32>().unwrap(),
            args[3].parse::<f32>().unwrap(),
        ),
        args[4].parse::<f32>().unwrap(),
        Vec2::new(
            args[5].parse::<f32>().unwrap(),
            args[6].parse::<f32>().unwrap(),
        ),
        args[7].parse::<f32>().unwrap(),
    )?;

    /// This data can also be written direct to the USB device by transforming with
    let data = AniMePacketType::from(image);
    let data = <AniMeDataBuffer>::from(&image);
    client
        .proxies()
        .anime()
        .write(data)
        .unwrap();

    Ok(())
}
```
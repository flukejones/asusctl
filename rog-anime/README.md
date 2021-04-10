# rog-anime

`rog-anime` is a crate for use with ASUS laptops that have an AniMe matrix display built in to them. The crate can be used with zbus to communicate with the `asusd` daemon from the project this crate is part of, or it can be used standalone to write the data directly to USB by transforming the data to USB HID packets using builtin functions.

Supported so far is:
- Sequences of data,
- Data can be:
  + Image<scale, rotate, position>
  + ASUS style gif
  + Plain image type gif
  + Pause
- Create USB HID packets for writing to the device

This crate is mostly purpose built for use with [`asus-nb-ctrl`](https://gitlab.com/asus-linux/asus-nb-ctrl) which is a complete daemon and toolset for Linux on ASUS ROG/TUF machines, but can be used in general for example building a new controller for Windows OS.

## Feature enablement

`dbus` is enabled by default - this uses `zvariant` to enable sending some types over dbus interfaces.

## Example

```rust
use std::{
    env, error::Error, f32::consts::PI, path::Path, process::exit, thread::sleep, time::Duration,
};

use rog_anime::{
    AniMeDataBuffer, {AniMeImage, Vec2},
};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = AuraDbusClient::new().unwrap();

    let mut image = AniMeImage::from_png(
        Path::new("./doom.png"),
        0.9, // scale
        0.0, // rotation
        Vec2::new(0.0, 0.0), // position
        0.3, // brightness
    )?;

    loop {
        image.angle += 0.05;
        if image.angle > PI * 2.0 {
            image.angle = 0.0
        }
        image.update();

        client
            .proxies()
            .anime()
            .write(<AniMeDataBuffer>::from(&image))
            .unwrap();
        sleep(Duration::from_micros(500));
    }
}
```

## Example, USB HID
```rust
let mut image = AniMeImage::from_png(
        Path::new("./doom.png"),
        0.9, // scale
        0.0, // rotation
        Vec2::new(0.0, 0.0), // position
        0.3, // brightness
    )?;

// convert to intermediate packet format
let buffer = <AniMeDataBuffer>::from(&image)
// then to USB HID
let data = AniMePacketType::from(buffer);
// and then write direct
for packet in data.iter() {
    write_usb(packet); // some usb call here
}
```

## data

- `data/controller.gif` is an example ASUS diagonally orientated gif.
- `data/diagonal-template.*` are templates for diagonal images or gifs.

See https://blog.joshwalsh.me/asus-anime-matrix/ for details on how
the diagonal layout works.

`diagonal-template.*` is provided from the website above. It is best to
export the final file to 36px height - no scaling is done in asusd or
rog-anime crate for diagonal displays.

## TODO:
- Diagonal font and text
- General font and text
- System info for Linux and Windows
- Audio EQ visual for Linux and Windows
- Time+Date display
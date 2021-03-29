use std::{env, error::Error, path::Path, process::exit};

use rog_anime::{
    AniMeDataBuffer, {AnimeImage, Vec2},
};
use rog_dbus::AuraDbusClient;

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

    let matrix = AnimeImage::from_png(
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

    client
        .proxies()
        .anime()
        .write(<AniMeDataBuffer>::from(&matrix))
        .unwrap();

    Ok(())
}

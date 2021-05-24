use std::{env, error::Error, path::Path, process::exit};

use rog_anime::{
    AnimeDataBuffer, {AnimeImage, Vec2},
};
use rog_dbus::RogDbusClient;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = RogDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 7 {
        println!("Usage: <filepath> <scale> <angle> <x pos> <y pos> <brightness>");
        println!("e.g, asusctl/examples/doom_large.png 0.9 0.4 0.0 0.0 0.8");
        exit(-1);
    }

    let matrix = AnimeImage::from_png(
        Path::new(&args[1]),
        args[2].parse::<f32>().unwrap(),
        args[3].parse::<f32>().unwrap(),
        Vec2::new(
            args[4].parse::<f32>().unwrap(),
            args[5].parse::<f32>().unwrap(),
        ),
        args[6].parse::<f32>().unwrap(),
    )?;

    client
        .proxies()
        .anime()
        .write(<AnimeDataBuffer>::from(&matrix))
        .unwrap();

    Ok(())
}

use std::{env, error::Error, path::Path, process::exit};

use rog_anime::{usb::get_anime_type, AnimeDiagonal, AnimeType};
use rog_dbus::RogDbusClientBlocking;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = RogDbusClientBlocking::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!("Usage: <filepath> <brightness>");
        println!("e.g, asusctl/examples/doom_large.png 0.8");
        exit(-1);
    }

    let matrix = AnimeDiagonal::from_png(
        Path::new(&args[1]),
        None,
        args[2].parse::<f32>().unwrap(),
        AnimeType::GA401,
    )?;

    let anime_type = get_anime_type()?;

    client
        .proxies()
        .anime()
        .write(matrix.into_data_buffer(anime_type)?)
        .unwrap();

    Ok(())
}

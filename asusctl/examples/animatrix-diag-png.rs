use std::{env, error::Error, path::Path, process::exit};

use rog_anime::{AnimeDataBuffer, AnimeDiagonal};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!("Usage: <filepath> <brightness>");
        println!("e.g, asusctl/examples/doom_large.png 0.8");
        exit(-1);
    }

    let matrix =
        AnimeDiagonal::from_png(Path::new(&args[1]), None, args[2].parse::<f32>().unwrap())?;

    client
        .proxies()
        .anime()
        .write(<AnimeDataBuffer>::from(&matrix))
        .unwrap();

    Ok(())
}

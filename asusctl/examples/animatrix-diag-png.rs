use std::{env, error::Error, path::Path, process::exit};

use rog_anime::{
    AniMeDataBuffer, {AniMeDiagonal, Vec2},
};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!(
            "Usage: <filepath> <brightness>"
        );
        println!("e.g, asusctl/examples/doom_large.png 0.8");
        exit(-1);
    }

    let matrix = AniMeDiagonal::from_png(
        Path::new(&args[1]),
        args[2].parse::<f32>().unwrap(),
    )?;

    client
        .proxies()
        .anime()
        .write(<AniMeDataBuffer>::from(&matrix))
        .unwrap();

    Ok(())
}

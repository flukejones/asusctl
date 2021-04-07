use std::{env, path::Path, thread::sleep};

use glam::Vec2;
use rog_anime::AniMeBlock;
use rog_dbus::AuraDbusClient;

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 7 {
        println!("Usage: <filepath> <scale> <angle> <x pos> <y pos> <brightness>");
        println!("e.g, asusctl/examples/file.gif 0.9 0.4 0.0 0.0 0.8");
        return;
    }

    let gif = AniMeBlock::image_gif(
        Path::new(&args[1]),
        args[2].parse::<f32>().unwrap(),
        args[3].parse::<f32>().unwrap(),
        Vec2::new(
            args[4].parse::<f32>().unwrap(),
            args[5].parse::<f32>().unwrap(),
        ),
        args[6].parse::<f32>().unwrap(),
    )
    .unwrap();

    loop {
        for frame in gif.get_animation().unwrap().frames() {
            client
                .proxies()
                .anime()
                .write(frame.frame().clone())
                .unwrap();
            sleep(frame.delay());
        }
    }
}

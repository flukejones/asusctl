use std::{env, path::Path, thread::sleep};

use rog_anime::{ActionData, ActionLoader, Sequences};
use rog_dbus::RogDbusClient;

fn main() {
    let (client, _) = RogDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!("Please supply filepath and brightness");
        return;
    }

    let path = Path::new(&args[1]);
    let brightness = args[2].parse::<f32>().unwrap();
    let mut seq = Sequences::new();
    seq.insert(
        0,
        &ActionLoader::AsusAnimation {
            file: path.into(),
            time: rog_anime::AnimTime::Infinite,
            brightness,
        },
    )
    .unwrap();

    loop {
        for action in seq.iter() {
            if let ActionData::Animation(frames) = action {
                for frame in frames.frames() {
                    client
                        .proxies()
                        .anime()
                        .write(frame.frame().clone())
                        .unwrap();
                    sleep(frame.delay());
                }
            }
        }
    }
}

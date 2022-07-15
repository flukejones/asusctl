use std::{env, path::Path, thread::sleep};

use rog_anime::{usb::get_anime_type, ActionData, ActionLoader, Sequences};
use rog_dbus::RogDbusClientBlocking;

fn main() {
    let (client, _) = RogDbusClientBlocking::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!("Please supply filepath and brightness");
        return;
    }

    let path = Path::new(&args[1]);
    let brightness = args[2].parse::<f32>().unwrap();
    let anime_type = get_anime_type().unwrap();
    let mut seq = Sequences::new(anime_type);
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

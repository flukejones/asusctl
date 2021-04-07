use std::{env, path::Path, thread::sleep};

use rog_anime::AniMeBlock;
use rog_dbus::AuraDbusClient;

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 3 {
        println!("Please supply filepath and brightness");
        return;
    }

    let path = Path::new(&args[1]);
    let brightness = args[2].parse::<f32>().unwrap();
    let gif = AniMeBlock::asus_gif(path, brightness).unwrap();

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

use std::{env, path::Path, thread::sleep};

use rog_anime::{Action, Sequences};
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
    let mut seq = Sequences::new();
    seq.add_asus_gif(path, None, brightness).unwrap();

    loop {
        for action in seq.iter() {
            if let Action::Animation(frames) = action {
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

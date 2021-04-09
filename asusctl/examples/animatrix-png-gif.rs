use std::{
    env,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use glam::Vec2;
use rog_anime::{Action, AnimTime, Sequences};
use rog_dbus::AuraDbusClient;

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() < 7 {
        println!(
            "Usage: <filepath> <scale> <angle> <x pos> <y pos> <brightness> <duration> <filepath>"
        );
        println!("e.g, asusctl/examples/file.gif 0.9 0.4 0.0 0.0 0.8 0");
        return;
    }

    let mut seq = Sequences::new();
    seq.add_image_gif(
        Path::new(&args[1]),
        args[2].parse::<f32>().unwrap(),
        args[3].parse::<f32>().unwrap(),
        Vec2::new(
            args[4].parse::<f32>().unwrap(),
            args[5].parse::<f32>().unwrap(),
        ),
        if let Ok(time) = args[7].parse::<u64>() {
            if time != 0 {
                AnimTime::Time(Duration::from_secs(time))
            } else {
                AnimTime::Infinite
            }
        } else {
            AnimTime::Infinite
        },
        args[6].parse::<f32>().unwrap(),
    )
    .unwrap();

    if args.len() == 9 {
        seq.add_image_gif(
            Path::new(&args[8]),
            args[2].parse::<f32>().unwrap(),
            args[3].parse::<f32>().unwrap(),
            Vec2::new(
                args[4].parse::<f32>().unwrap(),
                args[5].parse::<f32>().unwrap(),
            ),
            if let Ok(time) = args[7].parse::<u64>() {
                if time != 0 {
                    AnimTime::Time(Duration::from_secs(time))
                } else {
                    AnimTime::Infinite
                }
            } else {
                AnimTime::Infinite
            },
            args[6].parse::<f32>().unwrap(),
        )
        .unwrap();
    }

    loop {
        for action in seq.iter() {
            if let Action::Animation(frames) = action {
                let mut count = 0;
                let start = Instant::now();
                'outer: loop {
                    for frame in frames.frames() {
                        client
                            .proxies()
                            .anime()
                            .write(frame.frame().clone())
                            .unwrap();
                        if let AnimTime::Time(time) = frames.duration() {
                            if Instant::now().duration_since(start) > time {
                                break 'outer;
                            }
                        } else if let AnimTime::Cycles(times) = frames.duration() {
                            if count == times {
                                break 'outer;
                            }
                        }
                        sleep(frame.delay());
                    }
                    count +=1;
                }
            }
        }
    }
}

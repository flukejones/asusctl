use std::{
    env, error::Error, f32::consts::PI, path::Path, process::exit, thread::sleep, time::Duration,
};

use rog_anime::{
    AniMeDataBuffer, {AniMeImage, Vec2},
};
use rog_dbus::AuraDbusClient;

fn main() -> Result<(), Box<dyn Error>> {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() != 8 {
        println!("Usage: <filepath> <x scale> <y scale> <angle> <x pos> <y pos> <brightness>");
        println!("e.g, asusctl/examples/doom_large.bmp 0.9 0.9 0.4 0.0 0.0, 0.8");
        println!("All args except path and fineness are floats");
        exit(-1);
    }

    let mut matrix = AniMeImage::from_png(
        Path::new(&args[1]),
        Vec2::new(
            args[2].parse::<f32>().unwrap(),
            args[3].parse::<f32>().unwrap(),
        ),
        args[4].parse::<f32>().unwrap(),
        Vec2::new(
            args[5].parse::<f32>().unwrap(),
            args[6].parse::<f32>().unwrap(),
        ),
        args[7].parse::<f32>().unwrap(),
    )?;

    loop {
        matrix.angle += 0.05;
        if matrix.angle > PI * 2.0 {
            matrix.angle = 0.0
        }
        matrix.update();

        client
            .proxies()
            .anime()
            .write(<AniMeDataBuffer>::from(&matrix))
            .unwrap();
        sleep(Duration::from_micros(500));
    }
}

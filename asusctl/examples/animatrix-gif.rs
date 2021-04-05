use std::{env, fs::File, path::Path, thread::sleep, time::Duration};

use rog_anime::{AniMeDataBuffer, AniMeDiagonal};
use rog_dbus::AuraDbusClient;

struct AniMeFrame {
    data: AniMeDataBuffer,
    delay: Duration,
}

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    let args: Vec<String> = env::args().into_iter().collect();
    // if args.len() != 7 {
    //     exit(-1);
    // }

    let mut frames = Vec::new();
    let mut matrix = AniMeDiagonal::new();

    let mut decoder = gif::DecodeOptions::new();
    // Configure the decoder such that it will expand the image to RGBA.
    decoder.set_color_output(gif::ColorOutput::RGBA);
    // Read the file header
    let file = File::open(Path::new(&args[1])).unwrap();
    let mut decoder = decoder.read_info(file).unwrap();

    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let wait = frame.delay;
        for (y, row) in frame.buffer.chunks(frame.width as usize * 4).enumerate() {
            for (x, px) in row.chunks(4).enumerate() {
                if px[3] != 255 {
                    // should be t but not in some gifs? What, ASUS, what?
                    continue;
                }
                matrix.get_mut()[y + frame.top as usize][x + frame.left as usize] = px[0];
            }
        }
        client
            .proxies()
            .anime()
            .write(<AniMeDataBuffer>::from(&matrix))
            .unwrap();

        frames.push(AniMeFrame {
            data: <AniMeDataBuffer>::from(&matrix),
            delay: Duration::from_millis(wait as u64),
        });

        sleep(Duration::from_millis(wait as u64));
    }

    loop {
        for frame in frames.iter() {
            client.proxies().anime().write(frame.data.clone()).unwrap();
            sleep(frame.delay);
        }
    }
}

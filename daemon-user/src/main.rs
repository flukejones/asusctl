use rog_anime::{Action, AnimTime};
use rog_dbus::AuraDbusClient;
use rog_user::user_config::*;

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("   rog-dbus version {}", rog_dbus::VERSION);

    let (client, _) = AuraDbusClient::new().unwrap();

    let mut config = UserConfig::new();
    config.load_config()?;
    let anime = config.create_anime()?;

    // TODO:
    //  - find user config dir with xdg
    //  - load user config
    //  - start anime
    // A way to reload when the config changes

    loop {
        for action in anime.iter() {
            let start = Instant::now();

            match action {
                Action::Animation(frames) => {
                    let mut count = 0;
                    'animation: loop {
                        for frame in frames.frames() {
                            client.proxies().anime().write(frame.frame().clone())?;
                            if let AnimTime::Time(time) = frames.duration() {
                                if Instant::now().duration_since(start) > time {
                                    break 'animation;
                                }
                            }
                            sleep(frame.delay());
                        }
                        if let AnimTime::Cycles(times) = frames.duration() {
                            count += 1;
                            if count >= times {
                                break 'animation;
                            }
                        }
                    }
                }
                Action::Image(image) => {
                    client.proxies().anime().write(image.as_ref().clone())?;
                }
                Action::Pause(duration) => 'pause: loop {
                    if Instant::now().duration_since(start) > *duration {
                        break 'pause;
                    }
                    sleep(Duration::from_millis(10));
                },
                Action::AudioEq => {}
                Action::SystemInfo => {}
                Action::TimeDate => {}
                Action::Matrix => {}
            }
        }
    }
}

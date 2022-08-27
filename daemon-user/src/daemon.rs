use rog_anime::usb::get_anime_type;
use rog_aura::layouts::KeyLayout;
use rog_dbus::RogDbusClientBlocking;
use rog_user::{
    ctrl_anime::{CtrlAnime, CtrlAnimeInner},
    user_config::*,
    DBUS_NAME,
};
use smol::Executor;
use std::sync::Mutex;
use std::{fs::OpenOptions, io::Read, path::PathBuf, sync::Arc};
use zbus::Connection;

use std::sync::atomic::AtomicBool;

#[cfg(not(feature = "local_data"))]
const DATA_DIR: &str = "/usr/share/rog-gui/";
#[cfg(feature = "local_data")]
const DATA_DIR: &str = env!("CARGO_MANIFEST_DIR");
const BOARD_NAME: &str = "/sys/class/dmi/id/board_name";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("  user daemon v{}", rog_user::VERSION);
    println!("    rog-anime v{}", rog_anime::VERSION);
    println!("     rog-dbus v{}", rog_dbus::VERSION);
    println!("rog-platform v{}", rog_platform::VERSION);

    let (client, _) = RogDbusClientBlocking::new()?;
    let supported = client.proxies().supported().supported_functions()?;

    let mut config = UserConfig::new();
    config.load()?;

    let executor = Executor::new();

    let early_return = Arc::new(AtomicBool::new(false));
    // Set up the anime data and run loop/thread
    if supported.anime_ctrl.0 {
        if let Some(cfg) = config.active_anime {
            let anime_type = get_anime_type()?;
            let anime_config = UserAnimeConfig::load(cfg)?;
            let anime = anime_config.create(anime_type)?;
            let anime_config = Arc::new(Mutex::new(anime_config));

            executor
                .spawn(async move {
                    // Create server
                    let mut connection = Connection::session().await.unwrap();
                    connection.request_name(DBUS_NAME).await.unwrap();

                    // Inner behind mutex required for thread safety
                    let inner = Arc::new(Mutex::new(
                        CtrlAnimeInner::new(anime, client, early_return.clone()).unwrap(),
                    ));
                    // Need new client object for dbus control part
                    let (client, _) = RogDbusClientBlocking::new().unwrap();
                    let anime_control =
                        CtrlAnime::new(anime_config, inner.clone(), client, early_return).unwrap();
                    anime_control.add_to_server(&mut connection).await;
                    loop {
                        if let Ok(inner) = inner.clone().try_lock() {
                            inner.run().ok();
                        }
                    }
                })
                .detach();
        }
    }

    // if supported.keyboard_led.per_key_led_mode {
    if let Some(cfg) = config.active_aura {
        let mut aura_config = UserAuraConfig::load(cfg)?;

        // Find and load a matching layout for laptop
        let mut file = OpenOptions::new()
            .read(true)
            .open(PathBuf::from(BOARD_NAME))
            .map_err(|e| {
                println!("{BOARD_NAME}, {e}");
                e
            })?;
        let mut board_name = String::new();
        file.read_to_string(&mut board_name)?;

        let layout = KeyLayout::find_layout(board_name.as_str(), PathBuf::from(DATA_DIR))
            .map_err(|e| {
                println!("{BOARD_NAME}, {e}");
            })
            .unwrap_or(KeyLayout::ga401_layout());

        executor
            .spawn(async move {
                // Create server
                let (client, _) = RogDbusClientBlocking::new().unwrap();
                // let connection = Connection::session().await.unwrap();
                // connection.request_name(DBUS_NAME).await.unwrap();

                loop {
                    aura_config.aura.next_state(&layout);
                    let packets = aura_config.aura.create_packets();

                    client.proxies().led().per_key_raw(packets).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(60));
                }
            })
            .detach();
    }
    // }

    loop {
        smol::block_on(executor.tick());
    }
}

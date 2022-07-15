use rog_anime::usb::get_anime_type;
use rog_dbus::RogDbusClientBlocking;
use rog_user::{
    ctrl_anime::{CtrlAnime, CtrlAnimeInner},
    user_config::*,
    DBUS_NAME,
};
use smol::Executor;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::Connection;

use std::sync::atomic::AtomicBool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("  user daemon v{}", rog_user::VERSION);
    println!("    rog-anime v{}", rog_anime::VERSION);
    println!("     rog-dbus v{}", rog_dbus::VERSION);
    println!("rog-supported v{}", rog_supported::VERSION);

    let (client, _) = RogDbusClientBlocking::new()?;
    let supported = client.proxies().supported().supported_functions()?;

    let mut config = UserConfig::new();
    config.load_config()?;

    let executor = Executor::new();

    let early_return = Arc::new(AtomicBool::new(false));
    // Set up the anime data and run loop/thread
    if supported.anime_ctrl.0 {
        let anime_type = get_anime_type()?;
        let anime_config = UserAnimeConfig::load_config(config.active_anime)?;
        let anime = anime_config.create_anime(anime_type)?;
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

    // if supported.keyboard_led.per_key_led_mode {
    //     executor
    //         .spawn(async move {
    //            //
    //         })
    //         .detach();
    // }

    loop {
        smol::block_on(executor.tick());
    }
}

use futures::executor::ThreadPool;
use rog_dbus::RogDbusClientBlocking;
use rog_user::{
    ctrl_anime::{CtrlAnime, CtrlAnimeInner},
    user_config::*,
    DBUS_NAME,
};
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

    let anime_config = UserAnimeConfig::load_config(config.active_anime)?;
    let anime = anime_config.create_anime()?;

    let anime_config = Arc::new(Mutex::new(anime_config));
    let thread_pool = ThreadPool::new().unwrap();

    let early_return = Arc::new(AtomicBool::new(false));
    thread_pool.spawn_ok(async move {
        // Create server
        let mut connection = Connection::session().await.unwrap();
        connection.request_name(DBUS_NAME).await.unwrap();

        // Set up the anime data and run loop/thread
        if supported.anime_ctrl.0 {
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
        }
        // future::ready(())
    });

    // if supported.keyboard_led.per_key_led_mode {}

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        // if let Err(err) = server.try_handle_next() {
        //     println!("{}", err);
        // }
    }
}

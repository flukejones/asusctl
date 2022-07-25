use rog_control_center::{
    config::Config, get_ipc_file, notify::start_notifications, on_tmp_dir_exists,
    page_states::PageDataStates, RogApp, SHOW_GUI,
};
use rog_dbus::RogDbusClientBlocking;
use std::{
    io::Read,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::spawn,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Startup
    let mut config = Config::load()?;
    let start_closed = config.startup_in_background;

    if config.startup_in_background {
        config.run_in_background = true;
        config.save()?;
    }

    let (dbus, _) = RogDbusClientBlocking::new().unwrap();
    let supported = dbus.proxies().supported().supported_functions().unwrap();
    // Cheap method to alert to notifications rather than spinning a thread for each
    // This is quite different when done in a retained mode app
    let charge_notified = Arc::new(AtomicBool::new(false));
    let bios_notified = Arc::new(AtomicBool::new(false));
    let aura_notified = Arc::new(AtomicBool::new(false));
    let anime_notified = Arc::new(AtomicBool::new(false));
    let profiles_notified = Arc::new(AtomicBool::new(false));
    let fans_notified = Arc::new(AtomicBool::new(false));
    let notifs_enabled = Arc::new(AtomicBool::new(config.enable_notifications));
    // TODO: change this to an error instead of the nested unwraps, then use to
    // display a bare box app with error message.
    let states = PageDataStates::new(
        notifs_enabled.clone(),
        charge_notified.clone(),
        bios_notified.clone(),
        aura_notified.clone(),
        anime_notified.clone(),
        profiles_notified.clone(),
        fans_notified.clone(),
        &supported,
        &dbus,
    );

    if config.enable_notifications {
        start_notifications(
            charge_notified,
            bios_notified,
            aura_notified,
            anime_notified,
            profiles_notified,
            fans_notified,
            notifs_enabled,
        )?;
    }

    // tmp-dir must live to the end of program life
    let _tmp_dir = match tempfile::Builder::new()
        .prefix("rog-gui")
        .rand_bytes(0)
        .tempdir()
    {
        Ok(tmp) => tmp,
        Err(_) => on_tmp_dir_exists().unwrap(),
    };

    let native_options = eframe::NativeOptions {
        decorated: true,
        ..Default::default()
    };

    let should_show_gui = Arc::new(AtomicBool::new(!start_closed));
    let should = should_show_gui.clone();
    spawn(move || {
        // Loop is blocked here until a single byte is read
        loop {
            let mut buf = [0u8; 4];
            if get_ipc_file().unwrap().read(&mut buf).is_ok() && buf[0] == SHOW_GUI {
                should_show_gui.store(true, Ordering::SeqCst);
                // Give the starting app a change to read or we'll race it
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    });

    eframe::run_native(
        "ROG Control Center",
        native_options,
        Box::new(move |cc| Box::new(RogApp::new(start_closed, config, should, states, cc))),
    );
}

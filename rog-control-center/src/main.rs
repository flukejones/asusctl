use rog_aura::layouts::KeyLayout;
use rog_control_center::{
    config::Config, get_ipc_file, notify::start_notifications, on_tmp_dir_exists,
    page_states::PageDataStates, RogApp, RogDbusClientBlocking, SHOW_GUI,
};

use std::{
    fs::{self, OpenOptions},
    io::Read,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::spawn,
    time::Duration,
};

#[cfg(not(feature = "mocking"))]
const DATA_DIR: &str = "/usr/share/rog-gui/";
#[cfg(feature = "mocking")]
const DATA_DIR: &str = env!("CARGO_MANIFEST_DIR");
const BOARD_NAME: &str = "/sys/class/dmi/id/board_name";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Startup
    let mut config = Config::load()?;
    let start_closed = config.startup_in_background;

    if config.startup_in_background {
        config.run_in_background = true;
        config.save()?;
    }

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

    let mut layout = KeyLayout::ga401_layout(); // default
    let mut path = PathBuf::from(DATA_DIR);
    #[cfg(feature = "mocking")]
    {
        board_name = "G513".to_string();
        path.pop();
        path.push("rog-aura");
        path.push("data");
    }
    path.push("layouts");
    let path = path.as_path();
    for p in fs::read_dir(path).map_err(|e| {
        println!("{:?}, {e}", path);
        e
    })? {
        let tmp = KeyLayout::from_file(&p?.path()).unwrap();
        if tmp.matches(board_name.as_str()) {
            layout = tmp;
            break;
        }
    }

    // Cheap method to alert to notifications rather than spinning a thread for each
    // This is quite different when done in a retained mode app
    let charge_notified = Arc::new(AtomicBool::new(false));
    let bios_notified = Arc::new(AtomicBool::new(false));
    let aura_notified = Arc::new(AtomicBool::new(false));
    let anime_notified = Arc::new(AtomicBool::new(false));
    let profiles_notified = Arc::new(AtomicBool::new(false));
    let fans_notified = Arc::new(AtomicBool::new(false));
    let notifs_enabled = Arc::new(AtomicBool::new(config.enable_notifications));

    let states = {
        let (dbus, _) = RogDbusClientBlocking::new()?;
        let supported = dbus.proxies().supported().supported_functions().unwrap();
        PageDataStates::new(
            layout,
            notifs_enabled.clone(),
            charge_notified.clone(),
            bios_notified.clone(),
            aura_notified.clone(),
            anime_notified.clone(),
            profiles_notified.clone(),
            fans_notified.clone(),
            &supported,
            &dbus,
        )? // TODO: if error, show alt GUI containing the error message
    };

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
        decorated: false,
        transparent: false,
        min_window_size: Some(egui::vec2(840.0, 600.0)),
        max_window_size: Some(egui::vec2(840.0, 600.0)),
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
        Box::new(move |cc| {
            Box::new(RogApp::new(start_closed, config, should, states, cc).unwrap())
        }),
    );
}

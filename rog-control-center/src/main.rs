use rog_aura::layouts::KeyLayout;
use rog_control_center::{
    config::Config, get_ipc_file, notify::start_notifications, on_tmp_dir_exists,
    page_states::PageDataStates, startup_error::AppErrorShow, RogApp, RogDbusClientBlocking,
    SHOWING_GUI, SHOW_GUI,
};

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

#[cfg(not(feature = "mocking"))]
const DATA_DIR: &str = "/usr/share/rog-gui/";
#[cfg(feature = "mocking")]
const DATA_DIR: &str = env!("CARGO_MANIFEST_DIR");
const BOARD_NAME: &str = "/sys/class/dmi/id/board_name";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        decorated: false,
        transparent: false,
        min_window_size: Some(egui::vec2(840.0, 600.0)),
        max_window_size: Some(egui::vec2(840.0, 600.0)),
        run_and_return: true,
        ..Default::default()
    };

    let (dbus, _) = RogDbusClientBlocking::new()
        .map_err(|e| {
            eframe::run_native(
                "ROG Control Center",
                native_options.clone(),
                Box::new(move |_| Box::new(AppErrorShow::new(e.to_string()))),
            );
        })
        .unwrap();

    // Startup
    let mut config = Config::load()?;
    let mut start_closed = config.startup_in_background;

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
        board_name = "gl504".to_string();
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

    start_notifications(
        charge_notified.clone(),
        bios_notified.clone(),
        aura_notified.clone(),
        anime_notified.clone(),
        profiles_notified.clone(),
        fans_notified.clone(),
        notifs_enabled.clone(),
    )?;

    // tmp-dir must live to the end of program life
    let _tmp_dir = match tempfile::Builder::new()
        .prefix("rog-gui")
        .rand_bytes(0)
        .tempdir()
    {
        Ok(tmp) => tmp,
        Err(_) => on_tmp_dir_exists().unwrap(),
    };

    loop {
        let states = {
            let supported = match dbus.proxies().supported().supported_functions() {
                Ok(s) => s,
                Err(e) => {
                    eframe::run_native(
                        "ROG Control Center",
                        native_options.clone(),
                        Box::new(move |_| Box::new(AppErrorShow::new(e.to_string()))),
                    );
                    return Ok(());
                }
            };

            PageDataStates::new(
                layout.clone(),
                notifs_enabled.clone(),
                charge_notified.clone(),
                bios_notified.clone(),
                aura_notified.clone(),
                anime_notified.clone(),
                profiles_notified.clone(),
                fans_notified.clone(),
                &supported,
                &dbus,
            )?
        };

        if !start_closed {
            let mut ipc_file = get_ipc_file().unwrap();
            ipc_file.write_all(&[SHOWING_GUI]).unwrap();
            eframe::run_native(
                "ROG Control Center",
                native_options.clone(),
                Box::new(move |cc| {
                    Box::new(RogApp::new(Config::load().unwrap(), states, cc).unwrap())
                }),
            );
        }

        let config = Config::load().unwrap();
        if !config.run_in_background {
            break;
        }

        let mut buf = [0u8; 4];
        // blocks until it is read, typically the read will happen after a second
        // process writes to the IPC (so there is data to actually read)
        if get_ipc_file().unwrap().read(&mut buf).is_ok() && buf[0] == SHOW_GUI {
            start_closed = false;
            continue;
        }
        dbg!("asda");
    }
    Ok(())
}

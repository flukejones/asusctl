use std::borrow::BorrowMut;
use std::env::args;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep, spawn};
use std::time::Duration;

use gumdrop::Options;
use log::LevelFilter;
use rog_control_center::cli_options::CliStart;
use rog_control_center::config::Config;
use rog_control_center::error::Result;
use rog_control_center::slint::ComponentHandle;
use rog_control_center::system_state::{AuraCreation, SystemState};
use rog_control_center::tray::init_tray;
use rog_control_center::update_and_notify::{start_notifications, EnabledNotifications};
use rog_control_center::{
    get_ipc_file, on_tmp_dir_exists, print_versions, MainWindow, RogDbusClientBlocking, QUIT_APP,
    SHOWING_GUI, SHOW_GUI,
};
use tokio::runtime::Runtime;
// use winit::monitor::VideoMode;
// use winit::window::{Fullscreen, WindowLevel};

fn main() -> Result<()> {
    // tmp-dir must live to the end of program life
    let _tmp_dir = match tempfile::Builder::new()
        .prefix("rog-gui")
        .rand_bytes(0)
        .tempdir()
    {
        Ok(tmp) => tmp,
        Err(_) => on_tmp_dir_exists().unwrap(),
    };

    let args: Vec<String> = args().skip(1).collect();

    let cli_parsed = match CliStart::parse_args_default(&args) {
        Ok(p) => p,
        Err(err) => {
            panic!("source {}", err);
        }
    };

    if do_cli_help(&cli_parsed) {
        return Ok(());
    }

    let mut logger = env_logger::Builder::new();
    logger
        .filter_level(LevelFilter::Warn)
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();

    // start tokio
    let rt = Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    let (dbus, _) = RogDbusClientBlocking::new()
        .map_err(|_| {
            // TODO: show an error window
        })
        .unwrap();

    let supported_properties = match dbus.proxies().platform().supported_properties() {
        Ok(s) => s,
        Err(_e) => {
            // TODO: show an error window
            Vec::default()
        }
    };

    // Startup
    let mut config = Config::load()?;

    if config.startup_in_background {
        config.run_in_background = true;
        let tmp = config.enabled_notifications.clone(); // ends up being a double clone, oh well.
        config.save(&tmp)?;
    } else {
        get_ipc_file().unwrap().write_all(&[SHOW_GUI, 0]).unwrap();
    }

    let enabled_notifications = EnabledNotifications::tokio_mutex(&config);
    let aura_creation = AuraCreation::new(cli_parsed.board_name, cli_parsed.layout_viewing)?;

    // TODO: config mutex to share config in various places
    let states = setup_page_state_and_notifs(aura_creation, &enabled_notifications, &config)?;

    if config.enable_tray_icon {
        init_tray(supported_properties, states.clone());
    }

    thread_local! { pub static UI: std::cell::RefCell<Option<MainWindow>> = Default::default()};
    i_slint_backend_selector::with_platform(|_| Ok(())).unwrap();

    let mut do_once = !config.startup_in_background;
    thread::spawn(move || {
        let mut buf = [0u8; 2];
        // blocks until it is read, typically the read will happen after a second
        // process writes to the IPC (so there is data to actually read)
        loop {
            if do_once {
                buf[0] = SHOW_GUI;
                do_once = false;
            } else {
                get_ipc_file().unwrap().read_exact(&mut buf).unwrap();
            }

            if buf[0] == SHOW_GUI {
                println!("Should show window {buf:?}");
                // There's a balancing act with read/write timing of IPC, there needs to be a
                // small sleep after this to give any other process a chance to
                // read the IPC before looping
                get_ipc_file()
                    .unwrap()
                    .write_all(&[SHOWING_GUI, 0])
                    .unwrap();
                sleep(Duration::from_millis(50));

                let states = states.clone();
                i_slint_core::api::invoke_from_event_loop(move || {
                    UI.with(|ui| {
                        let mut ui = ui.borrow_mut();
                        if let Some(ui) = ui.as_mut() {
                            ui.window().show().unwrap();
                            ui.window().on_close_requested(|| {
                                get_ipc_file().unwrap().write_all(&[0, 0]).unwrap();
                                slint::CloseRequestResponse::HideWindow
                            });
                        } else {
                            let newui = setup_window(states.clone());
                            newui.window().show().unwrap();
                            println!("New window should be showing now"); // but it isn't
                            newui.window().on_close_requested(|| {
                                get_ipc_file().unwrap().write_all(&[0, 0]).unwrap();
                                slint::CloseRequestResponse::HideWindow
                            });
                            ui.replace(newui);
                        }
                    });
                })
                .unwrap();
            } else if buf[1] == QUIT_APP {
                slint::quit_event_loop().unwrap();
            } else if buf[0] != SHOWING_GUI {
                Config::load().unwrap();
                if !config.run_in_background {
                    slint::quit_event_loop().unwrap();
                    return;
                }

                println!("Should hide window {buf:?}");
                i_slint_core::api::invoke_from_event_loop(move || {
                    UI.with(|ui| {
                        let mut ui = ui.take();
                        if let Some(ui) = ui.borrow_mut() {
                            ui.window().hide().unwrap();
                        }
                    });
                })
                .unwrap();
            }
        }
    });

    slint::run_event_loop_until_quit().unwrap();
    Ok(())
}

fn setup_window(_states: Arc<Mutex<SystemState>>) -> MainWindow {
    // slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().
    // unwrap())).unwrap();
    let ui = MainWindow::new().unwrap();
    // Example of how to do work in another thread.
    // The thread itself can keep its own state, and then update vars in the UI
    // when required.
    let ui_handle = ui.as_weak();
    spawn(move || loop {
        sleep(Duration::from_secs(1));
        // This is where the actual update happens
        ui_handle
            .upgrade_in_event_loop(move |handle| {
                // handle.set_counter(handle.get_counter() + 1);
                use i_slint_backend_winit::WinitWindowAccessor;
                handle
                    .window()
                    .with_winit_window(|winit_window: &winit::window::Window| {
                        // winit_window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        if !winit_window.has_focus() {
                            // slint::quit_event_loop().unwrap();
                            // handle.hide().unwrap();
                        }
                    });
            })
            .ok();
    });

    ui.on_exit_app(move || {
        slint::quit_event_loop().unwrap();
    });

    ui
}

fn setup_page_state_and_notifs(
    aura_creation: AuraCreation,
    enabled_notifications: &Arc<Mutex<EnabledNotifications>>,
    config: &Config,
) -> Result<Arc<Mutex<SystemState>>> {
    let page_states = Arc::new(Mutex::new(SystemState::new(
        aura_creation,
        enabled_notifications.clone(),
        config.enable_tray_icon,
        config.run_in_background,
    )?));

    start_notifications(config, &page_states, enabled_notifications)?;

    Ok(page_states)
}

// /// Bah.. the icon dosn't work on wayland anyway, but we'll leave it in for
// now. fn load_icon() -> IconData {
//     let path = PathBuf::from(APP_ICON_PATH);
//     let mut rgba = Vec::new();
//     let mut height = 512;
//     let mut width = 512;
//     if path.exists() {
//         if let Ok(data) = std::fs::read(path)
//             .map_err(|e| error!("Error reading app icon: {e:?}"))
//             .map_err(|e| error!("Error opening app icon: {e:?}"))
//         {
//             let data = std::io::Cursor::new(data);
//             let decoder = png_pong::Decoder::new(data).unwrap().into_steps();
//             let png_pong::Step { raster, delay: _ } =
// decoder.last().unwrap().unwrap();

//             if let png_pong::PngRaster::Rgba8(ras) = raster {
//                 rgba = ras.as_u8_slice().to_vec();
//                 width = ras.width();
//                 height = ras.height();
//                 info!("Loaded app icon. Not actually supported in Wayland
// yet");             }
//         }
//     } else {
//         error!("Missing {APP_ICON_PATH}");
//     }

//     IconData {
//         height,
//         width,
//         rgba,
//     }
// }

fn do_cli_help(parsed: &CliStart) -> bool {
    if parsed.help {
        println!("{}", CliStart::usage());
        println!();
        if let Some(cmdlist) = CliStart::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            for command in &commands {
                println!("{}", command);
            }
        }
    }

    if parsed.version {
        print_versions();
        println!();
    }

    parsed.help
}

pub fn get_layout_path(path: &Path, layout_name: &str) -> PathBuf {
    let mut data_path = PathBuf::from(path);
    let layout_file = format!("{}_US.ron", layout_name);
    data_path.push("layouts");
    data_path.push(layout_file);
    data_path
}
